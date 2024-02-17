use glam::{vec2, Vec2};

use crate::node::data::PhysicsProps;
use crate::physics::runge_kutta::{self, IsPhysicsVars, PhysicsState};
use crate::puppet::PuppetPhysics;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RigidPendulum {
	pub θ: f32,
	pub ω: f32,
}

impl IsPhysicsVars<2> for RigidPendulum {
	fn get_f32s(&self) -> [f32; 2] {
		[self.θ, self.ω]
	}

	fn set_f32s(&mut self, f32s: [f32; 2]) {
		[self.θ, self.ω] = f32s;
	}
}

impl PhysicsState<RigidPendulum> {
	pub(crate) fn tick(
		&mut self,
		puppet_physics: PuppetPhysics,
		props: &PhysicsProps,
		bob: Vec2,
		anchor: Vec2,
		dt: f32,
	) -> Vec2 {
		// Compute the angle against the updated anchor position
		let d_bob = bob - anchor;
		self.vars.θ = f32::atan2(-d_bob.x, d_bob.y);

		// Run the pendulum simulation in terms of angle
		runge_kutta::tick(&eval, self, (puppet_physics, props), anchor, dt);

		// Update the bob position at the new angle
		let angle = self.vars.θ;
		let d_bob = vec2(-angle.sin(), angle.cos());

		anchor + d_bob * props.length
	}
}

fn eval(
	state: &mut PhysicsState<RigidPendulum>,
	&(puppet_physics, props): &(PuppetPhysics, &PhysicsProps),
	_anchor: Vec2,
	_t: f32,
) {
	// https://www.myphysicslab.com/pendulum/pendulum-en.html

	let g = props.gravity * puppet_physics.pixels_per_meter * puppet_physics.gravity;
	let r = props.length;

	// θ' = ω
	state.derivatives.θ = state.vars.ω;

	// ω' = -(g/R) sin θ
	let dω = -(g / r) * state.vars.θ.sin();

	// critical damp: that way a damping value of 1 corresponds to no bouncing
	let crit_damp = 2. * (g / r).sqrt();

	let damping = -state.vars.ω * props.angle_damping * crit_damp;

	state.derivatives.ω = dω + damping;
}
