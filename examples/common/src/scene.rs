//! A nice scene controller to smoothly move around in the window.

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use glam::{vec2, Vec2};
use inox2d::math::camera::Camera;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};

pub struct ExampleSceneController {
	// for camera position and mouse interactions
	camera_pos: Vec2,
	mouse_pos: Vec2,
	mouse_pos_held: Vec2,
	mouse_state: ElementState,

	// for smooth scrolling
	pub scroll_speed: f32,
	hard_scale: Vec2,

	// for FPS-independent interactions
	start: Instant,
	prev_elapsed: f32,
	current_elapsed: f32,
}

impl ExampleSceneController {
	pub fn new(camera: &Camera, scroll_speed: f32) -> Self {
		Self {
			camera_pos: camera.position,
			mouse_pos: Vec2::default(),
			mouse_pos_held: Vec2::default(),
			mouse_state: ElementState::Released,
			scroll_speed,
			hard_scale: camera.scale,
			start: Instant::now(),
			prev_elapsed: 0.0,
			current_elapsed: 0.0,
		}
	}

	pub fn update(&mut self, camera: &mut Camera) {
		// Smooth scrolling
		let time_delta = self.current_elapsed - self.prev_elapsed;
		camera.scale = camera.scale + time_delta.powf(0.6) * (self.hard_scale - camera.scale);

		// Mouse dragging
		if self.mouse_state == ElementState::Pressed {
			camera.position = self.camera_pos + (self.mouse_pos - self.mouse_pos_held) / camera.scale;
		}

		// Frame interval
		self.prev_elapsed = self.current_elapsed;
		self.current_elapsed = self.start.elapsed().as_secs_f32();
	}

	pub fn interact(&mut self, event: &WindowEvent, camera: &Camera) {
		match event {
			WindowEvent::CursorMoved { position, .. } => {
				self.mouse_pos = vec2(position.x as f32, position.y as f32);
			}
			WindowEvent::MouseInput { state, .. } => {
				self.mouse_state = *state;
				if self.mouse_state == ElementState::Pressed {
					self.mouse_pos_held = self.mouse_pos;
					self.camera_pos = camera.position;
				}
			}
			WindowEvent::MouseWheel { delta, .. } => {
				// Handle mouse wheel (zoom)
				let my = match delta {
					MouseScrollDelta::LineDelta(_, y) => *y * 12.,
					MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
				};

				self.hard_scale *= 2_f32.powf(self.scroll_speed * my * 0.1);
			}
			_ => (),
		}
	}

	pub fn dt(&self) -> f32 {
		self.current_elapsed - self.prev_elapsed
	}

	pub fn current_elapsed(&self) -> f32 {
		self.current_elapsed
	}
}
