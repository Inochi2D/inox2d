#[cfg(target_arch = "wasm32")]
mod scene;

#[cfg(target_arch = "wasm32")]
fn create_window(event: &winit::event_loop::EventLoop<()>) -> Result<winit::window::Window, winit::error::OsError> {
	use winit::dpi::PhysicalSize;
	use winit::platform::web::WindowExtWebSys;
	use winit::window::WindowBuilder;

	let window = WindowBuilder::new()
		.with_resizable(false)
		.with_inner_size(PhysicalSize::new(1280, 720))
		.build(event)?;

	web_sys::window()
		.and_then(|win| win.document())
		.and_then(|doc| doc.body())
		.and_then(|body| {
			let canvas = web_sys::Element::from(window.canvas().unwrap());
			canvas.set_id("canvas");
			body.append_child(&canvas).ok()
		})
		.expect("couldn't append canvas to document body");

	Ok(window)
}

#[cfg(target_arch = "wasm32")]
fn request_animation_frame(f: &wasm_bindgen::prelude::Closure<dyn FnMut()>) {
	use wasm_bindgen::JsCast;
	web_sys::window()
		.unwrap()
		.request_animation_frame(f.as_ref().unchecked_ref())
		.expect("Couldn't register `requestAnimationFrame`");
}

#[cfg(target_arch = "wasm32")]
pub fn base_url() -> String {
	web_sys::window().unwrap().location().origin().unwrap()
}

#[cfg(target_arch = "wasm32")]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
	use std::cell::RefCell;
	use std::rc::Rc;

	use inox2d::{formats::inp::parse_inp, render::InoxRenderer};
	use inox2d_opengl::OpenglRenderer;

	use glam::Vec2;
	use tracing::info;
	use wasm_bindgen::prelude::Closure;
	use wasm_bindgen::JsCast;
	use winit::event::{Event, WindowEvent};
	use winit::platform::web::EventLoopExtWebSys;

	use crate::scene::WasmSceneController;

	let events = winit::event_loop::EventLoop::new().unwrap();
	let window = create_window(&events)?;

	// Make sure the context has a stencil buffer
	let context_options = js_sys::Object::new();
	js_sys::Reflect::set(&context_options, &"stencil".into(), &true.into()).unwrap();

	let gl = {
		let canvas = web_sys::window()
			.unwrap()
			.document()
			.unwrap()
			.get_element_by_id("canvas")
			.unwrap()
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.unwrap();
		let webgl2_context = canvas
			.get_context_with_context_options("webgl2", &context_options)
			.unwrap()
			.unwrap()
			.dyn_into::<web_sys::WebGl2RenderingContext>()
			.unwrap();
		glow::Context::from_webgl2_context(webgl2_context)
	};

	info!("Loading puppet");
	let res = reqwest::Client::new()
		.get(format!("{}/assets/puppet.inp", base_url()))
		.send()
		.await?;

	let model_bytes = res.bytes().await?;
	let model = parse_inp(model_bytes.as_ref())?;

	info!("Initializing Inox2D renderer");
	let window_size = window.inner_size();
	let mut renderer = OpenglRenderer::new(gl)?;

	info!("Creating buffers and uploading model textures");
	renderer.prepare(&model)?;
	renderer.resize(window_size.width, window_size.height);
	renderer.camera.scale = Vec2::splat(0.15);
	info!("Inox2D renderer initialized");

	let scene_ctrl = WasmSceneController::new(&renderer.camera, 0.5);

	// Refcells because we need to make our own continuous animation loop.
	// Winit won't help us :(
	let scene_ctrl = Rc::new(RefCell::new(scene_ctrl));
	let renderer = Rc::new(RefCell::new(renderer));
	let puppet = Rc::new(RefCell::new(model.puppet));

	// Setup continuous animation loop
	{
		let anim_loop_f = Rc::new(RefCell::new(None));
		let anim_loop_g = anim_loop_f.clone();
		let scene_ctrl = scene_ctrl.clone();
		let renderer = renderer.clone();
		let puppet = puppet.clone();

		*anim_loop_g.borrow_mut() = Some(Closure::new(move || {
			scene_ctrl.borrow_mut().update(&mut renderer.borrow_mut().camera);

			renderer.borrow().clear();
			{
				let mut puppet = puppet.borrow_mut();
				puppet.begin_set_params();
				let t = scene_ctrl.borrow().current_elapsed();
				puppet.set_param("Head:: Yaw-Pitch", Vec2::new(t.cos(), t.sin()));
				puppet.end_set_params();
			}
			renderer.borrow().render(&puppet.borrow());

			request_animation_frame(anim_loop_f.borrow().as_ref().unwrap());
		}));
		request_animation_frame(anim_loop_g.borrow().as_ref().unwrap());
	}

	// Event loop
	events.spawn(move |event, elwt| {
		// it needs to be present
		let _window = &window;
		elwt.set_control_flow(winit::event_loop::ControlFlow::Wait);

		match event {
			Event::WindowEvent { ref event, .. } => match event {
				WindowEvent::Resized(physical_size) => {
					// Handle window resizing
					renderer.borrow_mut().resize(physical_size.width, physical_size.height);
					window.request_redraw();
				}
				WindowEvent::CloseRequested => elwt.exit(),
				_ => scene_ctrl
					.borrow_mut()
					.interact(&window, event, &renderer.borrow().camera),
			},
			Event::AboutToWait => {
				window.request_redraw();
			}
			_ => (),
		}
	})?;

	Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn runwrap() {
	match run().await {
		Ok(_) => tracing::info!("Shutdown"),
		Err(e) => tracing::error!("Fatal crash: {}", e),
	}
}

#[cfg(target_arch = "wasm32")]
fn main() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
	wasm_bindgen_futures::spawn_local(runwrap());
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
	panic!("This is a WASM example. You need to build it for the WASM target.");
}
