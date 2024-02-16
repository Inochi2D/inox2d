use glam::{vec2, Vec2};

use crate::physics::runge_kutta::{self, IsPhysicsVars, PhysicsState};
use crate::physics::SimplePhysicsProps;
use crate::puppet::PuppetPhysics;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct RigidPendulum {
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

type RigidPendulumState = PhysicsState<RigidPendulum>;

fn eval(
	state: &mut RigidPendulumState,
	&(puppet_physics, props): &(PuppetPhysics, &SimplePhysicsProps),
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

#[derive(Debug, Clone, Default)]
pub struct RigidPendulumSystem {
	pub bob: Vec2,
	state: RigidPendulumState,
}

impl RigidPendulumSystem {
	pub fn tick(&mut self, anchor: Vec2, puppet_physics: PuppetPhysics, props: &SimplePhysicsProps, dt: f32) -> Vec2 {
		// Compute the angle against the updated anchor position
		let d_bob = self.bob - anchor;
		self.state.vars.θ = f32::atan2(-d_bob.x, d_bob.y);

		// Run the pendulum simulation in terms of angle
		runge_kutta::tick(&eval, &mut self.state, (puppet_physics, props), anchor, dt);

		// Update the bob position at the new angle
		let angle = self.state.vars.θ;
		let d_bob = vec2(-angle.sin(), angle.cos());
		self.bob = anchor + d_bob * props.length;

		self.bob
	}
}
