use std::path::PathBuf;
use std::{error::Error, fs};

use inox2d::animation::{Animation, AnimationAxis, AnimationLoopMode, AnimationTrack, Keyframe};
use inox2d::formats::inp::parse_inp;
use inox2d::math::interp::InterpolateMode;
use inox2d::model::Model;
use inox2d::render::InoxRendererExt;
use inox2d_opengl::OpenglRenderer;

use clap::Parser;
use glam::Vec2;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

use winit::event::{ElementState, KeyEvent, WindowEvent};

use common::scene::ExampleSceneController;
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::{KeyCode, PhysicalKey};

use app_frame::App;
use winit::window::WindowBuilder;

use crate::app_frame::AppFrame;

mod app_frame;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(help = "Path to the .inp or .inx file.")]
	inp_path: PathBuf,
}

/// Create demo animations for a puppet based on its available parameters.
/// This is useful for models that don't have embedded animations.
fn create_demo_animations(puppet: &mut inox2d::puppet::Puppet) {
	tracing::info!("Model has no embedded animations, creating demo animations...");
	tracing::info!("Available parameters:");
	for (name, param) in puppet.params().iter() {
		tracing::info!("  - {} (uuid: {}, is_vec2: {})", name, param.uuid.0, param.is_vec2);
	}

	// Collect parameter UUIDs first to avoid borrow conflicts
	let right_blink_uuid = puppet.params().get("Eye:: Right:: Blink").map(|p| p.uuid);
	let left_blink_uuid = puppet.params().get("Eye:: Left:: Blink").map(|p| p.uuid);
	let head_yaw_pitch_uuid = puppet.params().get("Head:: Yaw-Pitch").map(|p| p.uuid);
	let mouth_shape_uuid = puppet.params().get("Mouth:: Shape").map(|p| p.uuid);
	let breath_uuid = puppet.params().get("Breath").map(|p| p.uuid);

	let mut animations = Vec::new();

	// Create Blink animation if eye blink parameters exist
	if let Some(right_uuid) = right_blink_uuid {
		let mut tracks = vec![AnimationTrack {
			param_uuid: right_uuid,
			axis: AnimationAxis::X,
			keyframes: vec![
				Keyframe {
					time: 0.0,
					value: 0.0,
					interpolation: InterpolateMode::Linear,
				},
				Keyframe {
					time: 0.1,
					value: 1.0,
					interpolation: InterpolateMode::Linear,
				},
				Keyframe {
					time: 0.2,
					value: 0.0,
					interpolation: InterpolateMode::Linear,
				},
				Keyframe {
					time: 2.5,
					value: 0.0,
					interpolation: InterpolateMode::Linear,
				},
			],
		}];

		if let Some(left_uuid) = left_blink_uuid {
			tracks.push(AnimationTrack {
				param_uuid: left_uuid,
				axis: AnimationAxis::X,
				keyframes: vec![
					Keyframe {
						time: 0.0,
						value: 0.0,
						interpolation: InterpolateMode::Linear,
					},
					Keyframe {
						time: 0.1,
						value: 1.0,
						interpolation: InterpolateMode::Linear,
					},
					Keyframe {
						time: 0.2,
						value: 0.0,
						interpolation: InterpolateMode::Linear,
					},
					Keyframe {
						time: 2.5,
						value: 0.0,
						interpolation: InterpolateMode::Linear,
					},
				],
			});
		}

		animations.push(Animation {
			name: "Blink".to_string(),
			length: 2.5,
			loop_mode: AnimationLoopMode::Loop,
			tracks,
		});
		tracing::info!("Created 'Blink' animation");
	}

	// Create Idle Head animation if head parameters exist
	if let Some(head_uuid) = head_yaw_pitch_uuid {
		animations.push(Animation {
			name: "Idle Head".to_string(),
			length: 4.0,
			loop_mode: AnimationLoopMode::PingPong,
			tracks: vec![
				AnimationTrack {
					param_uuid: head_uuid,
					axis: AnimationAxis::X,
					keyframes: vec![
						Keyframe {
							time: 0.0,
							value: -0.3,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 2.0,
							value: 0.3,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 4.0,
							value: -0.3,
							interpolation: InterpolateMode::Linear,
						},
					],
				},
				AnimationTrack {
					param_uuid: head_uuid,
					axis: AnimationAxis::Y,
					keyframes: vec![
						Keyframe {
							time: 0.0,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 1.0,
							value: -0.2,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 2.0,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 3.0,
							value: 0.2,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 4.0,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
					],
				},
			],
		});
		tracing::info!("Created 'Idle Head' animation");
	}

	// Create Mouth animation if mouth parameters exist
	if let Some(mouth_uuid) = mouth_shape_uuid {
		animations.push(Animation {
			name: "Talking".to_string(),
			length: 0.6,
			loop_mode: AnimationLoopMode::Loop,
			tracks: vec![
				AnimationTrack {
					param_uuid: mouth_uuid,
					axis: AnimationAxis::X,
					keyframes: vec![
						Keyframe {
							time: 0.0,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.15,
							value: 0.5,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.3,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.45,
							value: 0.8,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.6,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
					],
				},
				AnimationTrack {
					param_uuid: mouth_uuid,
					axis: AnimationAxis::Y,
					keyframes: vec![
						Keyframe {
							time: 0.0,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.15,
							value: 0.3,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.3,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.45,
							value: 0.5,
							interpolation: InterpolateMode::Linear,
						},
						Keyframe {
							time: 0.6,
							value: 0.0,
							interpolation: InterpolateMode::Linear,
						},
					],
				},
			],
		});
		tracing::info!("Created 'Talking' animation");
	}

	// Create Breath animation if breath parameter exists
	if let Some(breath_u) = breath_uuid {
		animations.push(Animation {
			name: "Breathing".to_string(),
			length: 3.0,
			loop_mode: AnimationLoopMode::Loop,
			tracks: vec![AnimationTrack {
				param_uuid: breath_u,
				axis: AnimationAxis::X,
				keyframes: vec![
					Keyframe {
						time: 0.0,
						value: 0.0,
						interpolation: InterpolateMode::Linear,
					},
					Keyframe {
						time: 1.5,
						value: 1.0,
						interpolation: InterpolateMode::Linear,
					},
					Keyframe {
						time: 3.0,
						value: 0.0,
						interpolation: InterpolateMode::Linear,
					},
				],
			}],
		});
		tracing::info!("Created 'Breathing' animation");
	}

	// Add all animations to the puppet
	for anim in animations {
		puppet.add_animation(anim);
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let cli = Cli::parse();

	tracing_subscriber::registry()
		.with(fmt::layer())
		.with(LevelFilter::INFO)
		.init();

	tracing::info!("Parsing puppet");

	let data = fs::read(cli.inp_path)?;
	let mut model = parse_inp(data.as_slice())?;
	tracing::info!(
		"Successfully parsed puppet: {}",
		(model.puppet.meta.name.as_deref()).unwrap_or("<no puppet name specified in file>")
	);

	tracing::info!("Setting up puppet for transforms, params and rendering.");
	model.puppet.init_transforms();
	model.puppet.init_rendering();
	model.puppet.init_params();
	model.puppet.init_physics();

	// Create demo animations if the model has no embedded animations
	if model.puppet.animations.is_empty() {
		create_demo_animations(&mut model.puppet);
	}

	model.puppet.init_animations();
	tracing::info!("--- Animation Example ---");
	tracing::info!("Loaded {} animations:", model.puppet.animations.len());
	for (i, anim) in model.puppet.animations.iter().enumerate() {
		tracing::info!(
			"  [{}] {} ({:.2}s, {:?})",
			i + 1,
			anim.name,
			anim.length,
			anim.loop_mode
		);
	}
	tracing::info!("  [0] Stop all animations");
	tracing::info!("Press keys 1-9 to play animations, or 0 to stop.");
	tracing::info!("-------------------------");

	if let Some(anim) = model.puppet.animations.first() {
		let name = anim.name.clone();
		tracing::info!("Playing default animation: {}", name);
		model.puppet.play_animation(&name, 1.0);
	}

	tracing::info!("Setting up windowing and OpenGL");
	let app_frame = AppFrame::init(
		WindowBuilder::new()
			.with_transparent(true)
			.with_resizable(true)
			.with_inner_size(winit::dpi::PhysicalSize::new(600, 800))
			.with_title("Render Inochi2D Puppet (OpenGL)"),
	)?;

	app_frame.run(Inox2dOpenglExampleApp::new(model))?;

	Ok(())
}

struct Inox2dOpenglExampleApp {
	on_window: Option<(OpenglRenderer, ExampleSceneController)>,
	model: Model,
	width: u32,
	height: u32,
}

impl Inox2dOpenglExampleApp {
	pub fn new(model: Model) -> Self {
		Self {
			on_window: None,
			model,
			width: 0,
			height: 0,
		}
	}
}

impl App for Inox2dOpenglExampleApp {
	fn resume_window(&mut self, gl: glow::Context) {
		match OpenglRenderer::new(gl, &self.model) {
			Ok(mut renderer) => {
				tracing::info!("Initializing Inox2D renderer");
				renderer.resize(self.width, self.height);
				renderer.camera.scale = Vec2::splat(0.15);
				tracing::info!("Inox2D renderer initialized");

				let scene_ctrl = ExampleSceneController::new(&renderer.camera, 0.5);
				self.on_window = Some((renderer, scene_ctrl));
			}
			Err(e) => {
				tracing::error!("{}", e);
				self.on_window = None;
			}
		}
	}

	fn resize(&mut self, width: i32, height: i32) {
		self.width = width as u32;
		self.height = height as u32;

		if let Some((renderer, _)) = &mut self.on_window {
			renderer.resize(self.width, self.height);
		}
	}

	fn draw(&mut self) {
		let Some((renderer, scene_ctrl)) = &mut self.on_window else {
			return;
		};

		tracing::debug!("Redrawing");
		scene_ctrl.update(&mut renderer.camera);

		renderer.clear();

		let puppet = &mut self.model.puppet;
		puppet.begin_frame();
		// Animation system will handle parameter updates via end_frame()
		// Just pass the delta time for physics and animation updates
		puppet.end_frame(scene_ctrl.dt());

		renderer.on_begin_draw(puppet);
		renderer.draw(puppet);
		renderer.on_end_draw(puppet);
	}

	fn handle_window_event(&mut self, event: WindowEvent, elwt: &EventLoopWindowTarget<()>) {
		match event {
			WindowEvent::CloseRequested => elwt.exit(),
			WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						state: ElementState::Pressed,
						physical_key: PhysicalKey::Code(KeyCode::Escape),
						..
					},
				..
			} => {
				tracing::info!("There is an Escape D:");
				elwt.exit();
			}
			WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						state: ElementState::Pressed,
						physical_key: PhysicalKey::Code(code),
						..
					},
				..
			} => {
				let puppet = &mut self.model.puppet;
				match code {
					KeyCode::Digit1 => play_anim_idx(puppet, 0),
					KeyCode::Digit2 => play_anim_idx(puppet, 1),
					KeyCode::Digit3 => play_anim_idx(puppet, 2),
					KeyCode::Digit4 => play_anim_idx(puppet, 3),
					KeyCode::Digit5 => play_anim_idx(puppet, 4),
					KeyCode::Digit6 => play_anim_idx(puppet, 5),
					KeyCode::Digit7 => play_anim_idx(puppet, 6),
					KeyCode::Digit8 => play_anim_idx(puppet, 7),
					KeyCode::Digit9 => play_anim_idx(puppet, 8),
					KeyCode::Digit0 => {
						tracing::info!("Stopping all animations");
						puppet.stop_all_animations();
					}
					_ => {
						if let Some((renderer, scene_ctrl)) = &mut self.on_window {
							scene_ctrl.interact(&event, &renderer.camera)
						}
					}
				}
			}
			event => {
				if let Some((renderer, scene_ctrl)) = &mut self.on_window {
					scene_ctrl.interact(&event, &renderer.camera)
				}
			}
		}
	}
}

fn play_anim_idx(puppet: &mut inox2d::puppet::Puppet, idx: usize) {
	if let Some(anim) = puppet.animations.get(idx) {
		let name = anim.name.clone();
		tracing::info!("Playing animation: {}", name);
		puppet.stop_all_animations();
		puppet.play_animation(&name, 1.0);
	}
}
