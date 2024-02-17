use crate::nodes::node_data::PhysicsProps;
use crate::physics::runge_kutta::{self, IsPhysicsVars, PhysicsState};
use crate::puppet::PuppetPhysics;
use glam::{vec2, Vec2};
use std::f32::consts::PI;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SpringPendulum {
	pub bob_pos: Vec2,
	pub bob_vel: Vec2,
}

impl PhysicsState<SpringPendulum> {
	pub(crate) fn tick(
		&mut self,
		puppet_physics: PuppetPhysics,
		props: &PhysicsProps,
		bob: Vec2,
		anchor: Vec2,
		dt: f32,
	) -> Vec2 {
		self.vars.bob_pos = bob;

		// Run the spring pendulum simulation
		runge_kutta::tick(&eval, self, (puppet_physics, props), anchor, dt);

		self.vars.bob_pos
	}
}

impl IsPhysicsVars<4> for SpringPendulum {
	fn get_f32s(&self) -> [f32; 4] {
		[self.bob_pos.x, self.bob_pos.y, self.bob_vel.x, self.bob_vel.y]
	}

	fn set_f32s(&mut self, f32s: [f32; 4]) {
		[self.bob_pos.x, self.bob_pos.y, self.bob_vel.x, self.bob_vel.y] = f32s;
	}
}

fn eval(
	state: &mut PhysicsState<SpringPendulum>,
	&(puppet_physics, props): &(PuppetPhysics, &PhysicsProps),
	anchor: Vec2,
	_t: f32,
) {
	state.derivatives.bob_pos = state.vars.bob_vel;

	// These are normalized vs. mass
	let spring_ksqrt = props.frequency * 2. * PI;
	let spring_k = spring_ksqrt.powi(2);

	let g = props.gravity * puppet_physics.pixels_per_meter * puppet_physics.gravity;
	let rest_length = props.length - g / spring_k;

	let off_pos = state.vars.bob_pos - anchor;
	let off_pos_norm = off_pos.normalize();

	let length_ratio = g / props.length;
	let crit_damp_angle = 2. * length_ratio.sqrt();
	let crit_damp_length = 2. * spring_ksqrt;

	let dist = anchor.distance(state.vars.bob_pos).abs();
	let force = vec2(0., g) - (off_pos_norm * (dist - rest_length) * spring_k);

	let d_bob = state.vars.bob_vel;
	let d_bob_rot = vec2(
		d_bob.x * off_pos_norm.y + d_bob.y * off_pos_norm.x,
		d_bob.y * off_pos_norm.y - d_bob.x * off_pos_norm.x,
	);

	let dd_bob_rot = -vec2(
		d_bob_rot.x * props.angle_damping * crit_damp_angle,
		d_bob_rot.y * props.length_damping * crit_damp_length,
	);

	let dd_bob_damping = vec2(
		dd_bob_rot.x * off_pos_norm.y - d_bob_rot.y * off_pos_norm.x,
		dd_bob_rot.y * off_pos_norm.y + d_bob_rot.x * off_pos_norm.x,
	);

	let dd_bob = force + dd_bob_damping;

	state.derivatives.bob_vel = dd_bob;
}
