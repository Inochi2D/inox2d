use std::path::PathBuf;
use std::{error::Error, fs, num::NonZeroU32};

use inox2d::formats::inp::parse_inp;
use inox2d::render::InoxRenderer;
use inox2d_opengl::OpenglRenderer;

use clap::Parser;
use glam::Vec2;
use glutin::surface::GlSurface;
use tracing::{debug, info};
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use common::scene::ExampleSceneController;
use opengl::{launch_opengl_window, App};

mod opengl;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(help = "Path to the .inp file. .inx files don't work!")]
	inp_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
	let cli = Cli::parse();

	tracing_subscriber::registry()
		.with(fmt::layer())
		.with(LevelFilter::INFO)
		.init();

	info!("Parsing puppet");

	let data = fs::read(cli.inp_path).unwrap();
	let model = parse_inp(data.as_slice()).unwrap();
	info!(
		"Successfully parsed puppet: {}",
		(model.puppet.meta.name.as_deref()).unwrap_or("<no puppet name specified in file>")
	);

	info!("Setting up windowing and OpenGL");
	let App {
		gl,
		gl_ctx,
		gl_surface,
		gl_display,
		events,
		window,
	} = launch_opengl_window()?;

	info!("Initializing Inox2D renderer");
	let window_size = window.inner_size();

	let mut renderer = OpenglRenderer::new(gl)?;
	renderer.prepare(&model)?;
	renderer.resize(window_size.width, window_size.height);
	renderer.camera.scale = Vec2::splat(0.15);
	info!("Inox2D renderer initialized");

	let mut scene_ctrl = ExampleSceneController::new(&renderer.camera, 0.5);
	let mut puppet = model.puppet;

	// Event loop
	events.run(move |event, _, control_flow| {
		// They need to be present
		let _gl_display = &gl_display;
		let _window = &window;

		control_flow.set_wait();

		match event {
			Event::RedrawRequested(_) => {
				debug!("Redrawing");
				scene_ctrl.update(&mut renderer.camera);

				renderer.clear();

				puppet.begin_set_params();
				let t = scene_ctrl.current_elapsed();
				puppet.set_param("Head:: Yaw-Pitch", Vec2::new(t.cos(), t.sin()));
				puppet.end_set_params();

				renderer.render(&puppet);

				gl_surface.swap_buffers(&gl_ctx).unwrap();
				window.request_redraw();
			}
			Event::WindowEvent { ref event, .. } => match event {
				WindowEvent::Resized(physical_size) => {
					// Handle window resizing
					renderer.resize(physical_size.width, physical_size.height);
					gl_surface.resize(
						&gl_ctx,
						NonZeroU32::new(physical_size.width).unwrap(),
						NonZeroU32::new(physical_size.height).unwrap(),
					);
					window.request_redraw();
				}
				WindowEvent::CloseRequested => control_flow.set_exit(),
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							virtual_keycode: Some(VirtualKeyCode::Escape),
							state: ElementState::Pressed,
							..
						},
					..
				} => {
					info!("There is an Escape D:");
					control_flow.set_exit();
				}
				_ => scene_ctrl.interact(&window, event, &renderer.camera),
			},
			Event::MainEventsCleared => {
				window.request_redraw();
			}
			_ => (),
		}
	})
}
