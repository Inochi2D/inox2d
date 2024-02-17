pub mod pendulum;
pub(crate) mod runge_kutta;

use std::f32::consts::PI;

use glam::{vec2, vec4, Vec2};

use crate::nodes::node_data::{InoxData, ParamMapMode, PhysicsModel, SimplePhysics};
use crate::puppet::{Puppet, PuppetPhysics};
use crate::render::NodeRenderCtx;

impl Puppet {
	/// Update the puppet's nodes' absolute transforms, by applying further displacements yielded by the physics system
	/// in response to displacements caused by parameter changes
	pub fn update_physics(&mut self, dt: f32, puppet_physics: PuppetPhysics) {
		for driver_uuid in self.drivers.clone() {
			let Some(driver) = self.nodes.get_node_mut(driver_uuid) else {
				continue;
			};

			let InoxData::SimplePhysics(ref mut system) = driver.data else {
				continue;
			};

			let nrc = &self.render_ctx.node_render_ctxs[&driver.uuid];

			let output = system.update(dt, puppet_physics, nrc);
			let param_uuid = system.param;
			self.set_param(param_uuid, output);
		}
	}
}

impl SimplePhysics {
	fn update(&mut self, dt: f32, puppet_physics: PuppetPhysics, node_render_ctx: &NodeRenderCtx) -> Vec2 {
		// Timestep is limited to 10 seconds.
		// If you're getting 0.1 FPS, you have bigger issues to deal with.
		let mut dt = dt.min(10.);

		let anchor = self.calc_anchor(node_render_ctx);

		// Minimum physics timestep: 0.01s
		while dt > 0.01 {
			self.tick(0.01, anchor, puppet_physics);
			dt -= 0.01;
		}

		self.tick(dt, anchor, puppet_physics);

		self.output = self.calc_output(anchor, node_render_ctx);

		self.output
	}

	fn tick(&mut self, dt: f32, anchor: Vec2, puppet_physics: PuppetPhysics) {
		// enum dispatch, fill the branches once other systems are implemented
		// as for inox2d, users are not expected to bring their own physics system,
		// no need to do dynamic dispatch with something like Box<dyn SimplePhysicsSystem>
		self.bob = match &mut self.model_type {
			PhysicsModel::RigidPendulum(state) => state.tick(puppet_physics, &self.props, self.bob, anchor, dt),
			PhysicsModel::SpringPendulum(state) => state.tick(puppet_physics, &self.props, self.bob, anchor, dt),
		};
	}

	fn calc_anchor(&self, node_render_ctx: &NodeRenderCtx) -> Vec2 {
		let anchor = match self.local_only {
			true => node_render_ctx.trans_offset.translation.extend(1.0),
			false => node_render_ctx.trans * vec4(0.0, 0.0, 0.0, 1.0),
		};

		vec2(anchor.x, anchor.y)
	}

	fn calc_output(&self, anchor: Vec2, node_render_ctx: &NodeRenderCtx) -> Vec2 {
		let oscale = self.props.output_scale;
		let output = self.output;

		// "Okay, so this is confusing. We want to translate the angle back to local space, but not the coordinates."
		// - Asahi Lina

		// Transform the physics output back into local space.
		// The origin here is the anchor. This gives us the local angle.
		let local_pos4 = match self.local_only {
			true => vec4(output.x, output.y, 0.0, 1.0),
			false => node_render_ctx.trans.inverse() * vec4(output.x, output.y, 0.0, 1.0),
		};

		let local_angle = vec2(local_pos4.x, local_pos4.y).normalize();

		// Figure out the relative length. We can work this out directly in global space.
		let relative_length = output.distance(anchor) / self.props.length;

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
}
