use glam::{vec2, Vec2};

use crate::node::components::{PhysicsProps, RigidPendulumCtx};
use crate::physics::{
	pendulum::Pendulum,
	runge_kutta::{IsPhysicsVars, PhysicsState},
	PuppetPhysics, SimplePhysicsProps,
};

/// Variables for Runge-Kutta method.
#[derive(Default)]
pub(crate) struct RigidPendulum {
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

impl Pendulum for RigidPendulumCtx {
	fn get_bob(&self) -> Vec2 {
		self.bob
	}

	fn set_bob(&mut self, bob: Vec2) {
		self.bob = bob;
	}

	fn tick(&mut self, props: &SimplePhysicsProps, anchor: Vec2, t: f32, dt: f32) -> Vec2 {
		// Compute the angle against the updated anchor position
		let d_bob = self.bob - anchor;
		self.state.vars.θ = f32::atan2(-d_bob.x, d_bob.y);

		// Run the pendulum simulation in terms of angle
		self.state.tick(&eval, (props.0, &props.1.props), &anchor, t, dt);

		// Update the bob position at the new angle
		let angle = self.state.vars.θ;
		let d_bob = vec2(-angle.sin(), angle.cos());

		anchor + d_bob * props.1.props.length
	}
}

/// Acceleration of bob caused by gravity.
fn eval(
	state: &mut PhysicsState<2, RigidPendulum>,
	(puppet_physics, props): &(&PuppetPhysics, &PhysicsProps),
	_anchor: &Vec2,
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
