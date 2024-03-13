use glam::Vec2;

use crate::params::ParamUuid;

/// If has this as a component, the node is capable of doing Inochi2D SimplePhysics simulations
pub struct SimplePhysics {
	pub param: ParamUuid,
	pub model_type: PhysicsModel,
	pub map_mode: ParamMapMode,
	pub props: PhysicsProps,
	/// Whether physics system listens to local transform only.
	pub local_only: bool,
}

pub enum PhysicsModel {
	RigidPendulum,
	SpringPendulum,
}

pub enum ParamMapMode {
	AngleLength,
	XY,
}

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
