pub mod pendulum;
mod runge_kutta;
mod simple_physics;

use crate::nodes::node_data::InoxData;
use crate::params::ParamUuid;
use crate::puppet::{Puppet, PuppetPhysics};

use glam::Vec2;

use self::pendulum::rigid::RigidPendulumSystem;
use self::pendulum::spring::SpringPendulumSystem;

/// Physics model to use for simple physics
#[derive(Debug, Clone)]
pub enum SimplePhysicsSystem {
	/// Rigid pendulum
	RigidPendulum(RigidPendulumSystem),

	// Springy pendulum
	SpringPendulum(SpringPendulumSystem),
}

impl SimplePhysicsSystem {
	pub fn new_rigid_pendulum() -> Self {
		Self::RigidPendulum(RigidPendulumSystem::default())
	}

	pub fn new_spring_pendulum() -> Self {
		Self::SpringPendulum(SpringPendulumSystem::default())
	}

	fn tick(&mut self, anchor: Vec2, puppet_physics: PuppetPhysics, props: &SimplePhysicsProps, dt: f32) -> Vec2 {
		// enum dispatch, fill the branches once other systems are implemented
		// as for inox2d, users are not expected to bring their own physics system,
		// no need to do dynamic dispatch with something like Box<dyn SimplePhysicsSystem>
		match self {
			SimplePhysicsSystem::RigidPendulum(system) => system.tick(anchor, puppet_physics, props, dt),
			SimplePhysicsSystem::SpringPendulum(system) => system.tick(anchor, puppet_physics, props, dt),
		}
	}
}

#[derive(Debug, Clone)]
pub struct SimplePhysicsProps {
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

impl Default for SimplePhysicsProps {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParamMapMode {
	AngleLength,
	XY,
}

#[derive(Debug, Clone)]
pub struct SimplePhysics {
	pub param: ParamUuid,

	pub system: SimplePhysicsSystem,
	pub map_mode: ParamMapMode,

	pub props: SimplePhysicsProps,

	/// Whether physics system listens to local transform only.
	pub local_only: bool,

	pub anchor: Vec2,
	pub output: Vec2,
}

impl SimplePhysics {
	pub fn tick(&mut self, dt: f32, puppet_physics: PuppetPhysics) {
		self.output = self.system.tick(self.anchor, puppet_physics, &self.props, dt);
	}
}

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
