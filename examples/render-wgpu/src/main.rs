use common::scene::ExampleSceneController;
use glam::{uvec2, vec2, Vec2};
use wgpu::CompositeAlphaMode;
use winit::{
	event::*,
	event_loop::EventLoop,
	keyboard::{KeyCode, PhysicalKey},
	window::WindowBuilder,
};

use inox2d::formats::inp::parse_inp;
use inox2d::model::Model;
use inox2d_wgpu::Renderer;
use std::fs;
use std::path::PathBuf;

use clap::Parser;

pub async fn run(model: Model) {
	let event_loop = EventLoop::new().unwrap();
	let window = WindowBuilder::new()
		.with_inner_size(winit::dpi::PhysicalSize::new(800, 800))
		.with_resizable(true)
		.with_transparent(true)
		.with_title("Render Inochi2D Puppet (WGPU)")
		.build(&event_loop)
		.unwrap();

	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
	let surface = instance.create_surface(&window).unwrap();
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
				label: None,
				required_features: wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
				required_limits: wgpu::Limits::default(),
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
		desired_maximum_frame_latency: 2,
	};
	surface.configure(&device, &config);

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

	event_loop
		.run(|event, elwt| match event {
			Event::WindowEvent {
				window_id: _,
				event: WindowEvent::RedrawRequested,
			} => {
				scene_ctrl.update(&mut renderer.camera);

				puppet.begin_set_params();
				let t = scene_ctrl.current_elapsed();
				let _ = puppet.set_named_param("Head:: Yaw-Pitch", vec2(t.cos(), t.sin()));
				puppet.end_set_params(scene_ctrl.dt());

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
	#[arg(help = "Path to the .inp or .inx file.")]
	inp_path: PathBuf,
}

fn main() {
	let cli = Cli::parse();

	let data = fs::read(cli.inp_path).unwrap();
	let model = parse_inp(data.as_slice()).unwrap();

	pollster::block_on(run(model));
}
