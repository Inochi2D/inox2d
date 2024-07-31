pub mod pendulum;
pub(crate) mod runge_kutta;

use glam::Vec2;

use crate::node::components::{SimplePhysics, TransformStore};

/// Global physics parameters for the puppet.
pub struct PuppetPhysics {
	pub pixels_per_meter: f32,
	pub gravity: f32,
}

type SimplePhysicsProps<'a> = (&'a PuppetPhysics, &'a SimplePhysics);

/// Components implementing this will be able to yield a parameter value every frame based on
/// - time history of transforms of the associated node.
/// - Physics simulation.
pub trait SimplePhysicsCtx {
	/// Type of input to the simulation.
	type Anchor;

	/// Convert node transform to input.
	fn calc_anchor(&self, props: &SimplePhysicsProps, transform: &TransformStore) -> Self::Anchor;
	/// Run one step of simulation given input.
	fn tick(&mut self, props: &SimplePhysicsProps, anchor: &Self::Anchor, t: f32, dt: f32);
	/// Convert simulation result into a parameter value to set.
	fn calc_output(&self, props: &SimplePhysicsProps, transform: &TransformStore, anchor: Self::Anchor) -> Vec2;
}

/// Auto implemented trait for all `impl SimplePhysicsCtx`.
trait SimplePhysicsCtxCommon {
	fn update(&mut self, props: &SimplePhysicsProps, transform: &TransformStore, t: f32, dt: f32) -> Vec2;
}

impl<T: SimplePhysicsCtx> SimplePhysicsCtxCommon for T {
	/// Run physics simulation for one frame given provided methods. Handle big `dt` problems.
	fn update(&mut self, props: &SimplePhysicsProps, transform: &TransformStore, t: f32, dt: f32) -> Vec2 {
		// Timestep is limited to 10 seconds.
		// If you're getting 0.1 FPS, you have bigger issues to deal with.
		let mut dt = dt.min(10.);

		let anchor = self.calc_anchor(props, transform);

		// Minimum physics timestep: 0.01s. If not satisfied, break simulation into steps.
		let mut t = t;
		while dt > 0. {
			self.tick(props, &anchor, t, dt.min(0.01));
			t += 0.01;
			dt -= 0.01;
		}

		self.calc_output(props, transform, anchor)
	}
}
