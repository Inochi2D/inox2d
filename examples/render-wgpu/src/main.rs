use clap::Parser;
use common::scene::ExampleSceneController;
use env_logger;
use inox2d::formats::inp::parse_inp;
use inox2d::math::camera::Camera;
use inox2d::render::InoxRendererExt;
use inox2d_wgpu::WgpuRenderer;
use log::*;
use pollster::block_on;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(help = "Path to the .inp or .inx file.")]
	inp_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
	let cli = Cli::parse();

	env_logger::init();

	let event_loop = EventLoop::new()?;

	info!("Loading {:?}", cli.inp_path);

	let data = fs::read(cli.inp_path)?;
	let mut model = parse_inp(data.as_slice())?;

	info!(
		"Successfully parsed puppet: {}",
		(model.puppet.meta.name.as_deref()).unwrap_or("<no puppet name specified in file>")
	);

	model.puppet.init_transforms();
	model.puppet.init_rendering();
	model.puppet.init_params();
	model.puppet.init_physics();

	let window = Arc::new(WindowBuilder::new().build(&event_loop).expect("valid window"));
	let mut renderer = block_on(WgpuRenderer::new(window.clone(), &model)).expect("valid renderer");
	let mut scene_controller = ExampleSceneController::new(&Camera::default(), 0.5);
	let camera = Camera::default();

	event_loop.set_control_flow(ControlFlow::Poll);
	event_loop.run(|event, event_loop| {
		match event {
			Event::WindowEvent {
				event: WindowEvent::Resized(new_size),
				..
			} => {
				if let Err(err) = renderer.resize(new_size.width, new_size.height) {
					error!("Resize failed: {}", err);
				}
			}
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				event_loop.exit();
			}
			Event::WindowEvent {
				event: WindowEvent::RedrawRequested,
				..
			} => {
				if let Err(err) = renderer.draw(&model.puppet) {
					error!("Draw failed: {}", err);
				}
			}
			Event::WindowEvent { event, .. } => {
				scene_controller.interact(&event, &camera);
			}
			Event::AboutToWait => {
				window.request_redraw();

				//TODO: Swapchain? Swapchain.
			}
			_ => {}
		}
	})?;

	Ok(())
}
