use glam::Vec2;

use crate::params::ParamUuid;
use crate::physics::{
	pendulum::{rigid::RigidPendulum, spring::SpringPendulum},
	runge_kutta::PhysicsState,
};

/// If has this as a component, the node is capable of doing Inochi2D SimplePhysics simulations
#[derive(Clone)]
pub struct SimplePhysics {
	pub param: ParamUuid,
	pub model_type: PhysicsModel,
	pub map_mode: ParamMapMode,
	pub props: PhysicsProps,
	/// Whether physics system listens to local transform only.
	pub local_only: bool,
}

#[derive(Clone)]
pub enum PhysicsModel {
	RigidPendulum,
	SpringPendulum,
}

#[derive(Clone)]
pub enum ParamMapMode {
	AngleLength,
	XY,
}

#[derive(Clone)]
pub struct PhysicsProps {
	/// Gravity scale (1.0 = puppet gravity)
	pub gravity: f32,
	/// Pendulum/spring rest length (pixels)
	pub length: f32,
	/// Resonant frequency (Hz)
	pub frequency: f32,
	/// Angular damping ratio
	pub angle_damping: f32,
	/// Length damping ratio
	pub length_damping: f32,
	pub output_scale: Vec2,
}

impl Default for PhysicsProps {
	fn default() -> Self {
		Self {
			gravity: 1.,
			length: 1.,
			frequency: 1.,
			angle_damping: 0.5,
			length_damping: 0.5,
			output_scale: Vec2::ONE,
		}
	}
}

/// Physical states for simulating a rigid pendulum.
#[derive(Default)]
pub(crate) struct RigidPendulumCtx {
	pub bob: Vec2,
	pub state: PhysicsState<2, RigidPendulum>,
}

/// Physical states for simulating a spring pendulum.
#[derive(Default)]
pub(crate) struct SpringPendulumCtx {
	pub state: PhysicsState<4, SpringPendulum>,
}
