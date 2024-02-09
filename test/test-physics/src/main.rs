use common::scene::ExampleSceneController;
use glam::{uvec2, vec2, Vec2};
use gluon::{import::add_extern_module, new_vm, primitive, ThreadExt};
use gluon_codegen::{Getable, Pushable, Trace, VmType};
use wgpu::CompositeAlphaMode;
use winit::{
	event::*,
	event_loop::EventLoop,
	keyboard::{KeyCode, PhysicalKey},
	window::WindowBuilder,
};

use inox2d::model::Model;
use inox2d::{formats::inp::parse_inp, puppet::Puppet};
use inox2d_wgpu::Renderer;
use std::path::PathBuf;
use std::{borrow::BorrowMut, fs};

use clap::Parser;
pub async fn run_with_script(model: Model, script: String) {
	let event_loop = EventLoop::new().unwrap();
	let window = WindowBuilder::new()
		.with_inner_size(winit::dpi::PhysicalSize::new(800, 800))
		.with_resizable(true)
		.with_transparent(true)
		.with_title("Render Inochi2D Puppet (WGPU)")
		.build(&event_loop)
		.unwrap();

	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
	let surface = unsafe { instance.create_surface(&window) }.unwrap();
	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		})
		.await
		.unwrap();

	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
				limits: wgpu::Limits::default(),
				label: None,
			},
			None,
		)
		.await
		.unwrap();

	// Fallback to first alpha mode if PreMultiplied is not supported
	let alpha_modes = surface.get_capabilities(&adapter).alpha_modes;
	let alpha_mode = if alpha_modes.contains(&CompositeAlphaMode::PreMultiplied) {
		CompositeAlphaMode::PreMultiplied
	} else {
		alpha_modes[0]
	};

	let mut config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: wgpu::TextureFormat::Bgra8Unorm,
		width: window.inner_size().width,
		height: window.inner_size().height,
		present_mode: wgpu::PresentMode::Fifo,
		alpha_mode,
		view_formats: Vec::new(),
	};
	surface.configure(&device, &config);
	let vm = new_vm();

	let mut renderer = Renderer::new(
		&device,
		&queue,
		wgpu::TextureFormat::Bgra8Unorm,
		&model,
		uvec2(window.inner_size().width, window.inner_size().height),
	);
	renderer.camera.scale = Vec2::splat(0.15);

	let mut scene_ctrl = ExampleSceneController::new(&renderer.camera, 0.5);
	let mut puppet = model.puppet;
	add_extern_module(&vm, "inox_test_framework", load_mpsa);
	//	println!("{:?}", puppet);
	event_loop
		.run(move |event, elwt| match event {
			Event::WindowEvent {
				window_id: _,
				event: WindowEvent::RedrawRequested,
			} => {
				scene_ctrl.update(&mut renderer.camera);

				puppet.begin_set_params();
				let t = scene_ctrl.current_elapsed();
				let seeded_script = format!("let seed = {}\n{}", t, script);
				let (param_set_args, _) = vm.run_expr::<ParamSetArgs>("m", &seeded_script).unwrap();

				set_test_param(puppet.borrow_mut(), param_set_args);
				//				puppet.set_param("Anchor Positioner", vec2(t.cos(), t.sin()));
				puppet.end_set_params();

				let output = surface.get_current_texture().unwrap();
				let view = (output.texture).create_view(&wgpu::TextureViewDescriptor::default());

				renderer.render(&queue, &device, &puppet, &view);
				output.present();
			}
			Event::WindowEvent { ref event, .. } => match event {
				WindowEvent::CloseRequested
				| WindowEvent::KeyboardInput {
					event:
						KeyEvent {
							//virtual_keycode: Some(VirtualKeyCode::Escape),
							state: ElementState::Pressed,
							physical_key: PhysicalKey::Code(KeyCode::Escape),
							..
						},
					..
				} => {
					elwt.exit();
				}
				WindowEvent::Resized(size) => {
					// Reconfigure the surface with the new size
					config.width = size.width;
					config.height = size.height;
					surface.configure(&device, &config);

					// Update the renderer's internal viewport
					renderer.resize(uvec2(size.width, size.height));

					// On macos the window needs to be redrawn manually after resizing
					window.request_redraw();
				}
				_ => scene_ctrl.interact(event, &renderer.camera),
			},
			Event::AboutToWait => {
				// RedrawRequested will only trigger once, unless we manually
				// request it.
				window.request_redraw();
			}
			_ => {}
		})
		.expect("Error in event loop")
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(
		help = "Path to the .inp file. .inx files don't work!",
		default_value = "./inochi2d-models/arrows-doublespringpendulum.inp"
	)]
	inp_path: PathBuf,
	#[arg(short, long, default_value = "./test/test-physics/physics_test.glu")]
	script_path: PathBuf,
}
pub fn make_param_set_args(name: &str, val_x: f32, val_y: f32) -> ParamSetArgs {
	ParamSetArgs {
		name: name.into(),
		val_x,
		val_y,
	}
}
fn load_mpsa(vm: &gluon::Thread) -> gluon::vm::Result<gluon::vm::ExternModule> {
	gluon::vm::ExternModule::new(vm, primitive!(3, make_param_set_args))
}
#[derive(VmType, Clone, Debug, Trace, Getable, Pushable)]
#[gluon_userdata(clone)]
pub struct ParamSetArgs {
	pub name: String,
	pub val_x: f32,
	pub val_y: f32,
}
fn set_test_param(puppet: &mut Puppet, ParamSetArgs { name, val_x, val_y }: ParamSetArgs) {
	puppet.set_param(&name, vec2(val_x, val_y));
}
fn main() {
	let cli = Cli::parse();
	let script = String::from_utf8(fs::read(cli.script_path).unwrap()).unwrap();
	let data = fs::read(cli.inp_path).unwrap();
	let model = parse_inp(data.as_slice()).unwrap();

	pollster::block_on(run_with_script(model, script));
}
