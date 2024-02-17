use std::f32::consts::PI;

use glam::{vec2, vec4, Vec2};

use super::{ParamMapMode, SimplePhysics, SimplePhysicsSystem};
use crate::puppet::PuppetPhysics;
use crate::render::NodeRenderCtx;

impl SimplePhysics {
	fn update_inputs(&mut self, node_render_ctx: &NodeRenderCtx) {
		let anchor_pos = match self.local_only {
			true => node_render_ctx.trans_offset.translation.extend(1.0),
			false => node_render_ctx.trans * vec4(0.0, 0.0, 0.0, 1.0),
		};

		self.anchor = vec2(anchor_pos.x, anchor_pos.y);
	}

	fn calc_outputs(&self, node_render_ctx: &NodeRenderCtx) -> Vec2 {
		let oscale = self.props.output_scale;

		// "Okay, so this is confusing. We want to translate the angle back to local space, but not the coordinates."
		// - Asahi Lina

		// Transform the physics output back into local space.
		// The origin here is the anchor. This gives us the local angle.
		let local_pos4 = match self.local_only {
			true => vec4(self.output.x, self.output.y, 0.0, 1.0),
			false => node_render_ctx.trans.inverse() * vec4(self.output.x, self.output.y, 0.0, 1.0),
		};

		let local_angle = vec2(local_pos4.x, local_pos4.y).normalize();

		// Figure out the relative length. We can work this out directly in global space.
		let relative_length = self.output.distance(self.anchor) / self.props.length;

		let param_value = match self.map_mode {
			ParamMapMode::XY => {
				let local_pos_norm = local_angle * relative_length;
				let mut result = local_pos_norm - Vec2::Y;
				result.y = -result.y; // Y goes up for params
				result
			}
			ParamMapMode::AngleLength => {
				let a = f32::atan2(-local_angle.x, local_angle.y) / PI;
				vec2(a, relative_length)
			}
		};

		param_value * oscale
	}

	pub fn update(&mut self, dt: f32, puppet_physics: PuppetPhysics, node_render_ctx: &NodeRenderCtx) -> Vec2 {
		// Timestep is limited to 10 seconds.
		// If you're getting 0.1 FPS, you have bigger issues to deal with.
		let mut h = dt.min(10.);

		self.update_inputs(node_render_ctx);

		// Minimum physics timestep: 0.01s
		while h > 0.01 {
			self.tick(0.01, puppet_physics);
			h -= 0.01;
		}

		self.tick(h, puppet_physics);

		self.calc_outputs(node_render_ctx)
	}

	pub fn update_anchor(&mut self) {
		let bob = self.anchor + vec2(0.0, self.props.length);

		match &mut self.system {
			SimplePhysicsSystem::RigidPendulum(system) => system.bob = bob,
			SimplePhysicsSystem::SpringPendulum(system) => system.bob = bob,
		}
	}
}
