pub mod meta;
mod transforms;
mod tree;
mod world;

use std::collections::HashMap;

use crate::node::{InoxNode, InoxNodeUuid};
use crate::params::{Param, ParamCtx};
use crate::physics::{PhysicsCtx, PuppetPhysics};
use crate::render::RenderCtx;

use meta::PuppetMeta;
use transforms::TransformCtx;
pub use tree::InoxNodeTree;
pub use world::World;

/// Inochi2D puppet.
pub struct Puppet {
	pub meta: PuppetMeta,
	physics: PuppetPhysics,
	physics_ctx: Option<PhysicsCtx>,
	pub(crate) nodes: InoxNodeTree,
	pub(crate) node_comps: World,
	/// Currently only a marker for if transform/zsort components are initialized.
	pub(crate) transform_ctx: Option<TransformCtx>,
	/// Context for rendering this puppet. See `.init_rendering()`.
	pub render_ctx: Option<RenderCtx>,
	pub(crate) params: HashMap<String, Param>,
	/// Context for animating puppet with parameters. See `.init_params()`
	pub param_ctx: Option<ParamCtx>,
}

impl Puppet {
	pub(crate) fn new(
		meta: PuppetMeta,
		physics: PuppetPhysics,
		root: InoxNode,
		params: HashMap<String, Param>,
	) -> Self {
		Self {
			meta,
			physics,
			physics_ctx: None,
			nodes: InoxNodeTree::new_with_root(root),
			node_comps: World::new(),
			transform_ctx: None,
			render_ctx: None,
			params,
			param_ctx: None,
		}
	}

	/// Create a copy of node transform/zsort for modification. Panicks on second call.
	pub fn init_transforms(&mut self) {
		if self.transform_ctx.is_some() {
			panic!("Puppet transforms already initialized.")
		}

		let transform_ctx = TransformCtx::new(self);
		self.transform_ctx = Some(transform_ctx);
	}

	/// Call this on a freshly loaded puppet if rendering is needed. Panicks:
	/// - if transforms are not initialized.
	/// - on second call.
	pub fn init_rendering(&mut self) {
		if self.transform_ctx.is_none() {
			panic!("Puppet rendering depends on initialized puppet transforms.");
		}
		if self.render_ctx.is_some() {
			panic!("Puppet already initialized for rendering.");
		}

		let render_ctx = RenderCtx::new(self);
		self.render_ctx = Some(render_ctx);
	}

	/// Call this on a puppet if params are going to be used. Panicks:
	/// - if rendering is not initialized.
	/// - on second call.
	pub fn init_params(&mut self) {
		if self.render_ctx.is_none() {
			panic!("Only a puppet initialized for rendering can be animated by params.");
		}
		if self.param_ctx.is_some() {
			panic!("Puppet already initialized for params.");
		}

		let param_ctx = ParamCtx::new(self);
		self.param_ctx = Some(param_ctx);
	}

	/// Call this on a puppet if physics are going to be simulated. Panicks:
	/// - if params is not initialized.
	/// - on second call.
	pub fn init_physics(&mut self) {
		if self.param_ctx.is_none() {
			panic!("Puppet physics depends on initialized puppet params.");
		}
		if self.physics_ctx.is_some() {
			panic!("Puppet already initialized for physics.");
		}

		let physics_ctx = PhysicsCtx::new(self);
		self.physics_ctx = Some(physics_ctx);
	}

	/// Prepare the puppet for a new frame. User may set params afterwards.
	pub fn begin_frame(&mut self) {
		if let Some(render_ctx) = self.render_ctx.as_mut() {
			render_ctx.reset(&self.nodes, &mut self.node_comps);
		}

		if let Some(transform_ctx) = self.transform_ctx.as_mut() {
			transform_ctx.reset(&self.nodes, &mut self.node_comps);
		}

		if let Some(param_ctx) = self.param_ctx.as_mut() {
			param_ctx.reset(&self.params);
		}
	}

	/// Freeze puppet for one frame. Rendering, if initialized, may follow.
	///
	/// Provide elapsed time for physics, if initialized, to run. Provide `0` for the first call.
	pub fn end_frame(&mut self, dt: f32) {
		if let Some(param_ctx) = self.param_ctx.as_mut() {
			param_ctx.apply(&self.params, &mut self.node_comps);
		}

		if let Some(transform_ctx) = self.transform_ctx.as_mut() {
			transform_ctx.update(&self.nodes, &mut self.node_comps);
		}

		if let Some(physics_ctx) = self.physics_ctx.as_mut() {
			let values_to_apply = physics_ctx.step(&self.physics, &self.nodes, &mut self.node_comps, dt);

			// TODO: Think about separating DeformStack reset and RenderCtx reset?
			self.render_ctx
				.as_mut()
				.expect("If physics is initialized, so does params, so does rendering.")
				.reset(&self.nodes, &mut self.node_comps);

			// TODO: Fewer repeated calculations of a same transform?
			let transform_ctx = self
				.transform_ctx
				.as_mut()
				.expect("If physics is initialized, so does transforms.");
			transform_ctx.reset(&self.nodes, &mut self.node_comps);

			let param_ctx = self
				.param_ctx
				.as_mut()
				.expect("If physics is initialized, so does params.");
			for (param_name, value) in &values_to_apply {
				param_ctx
					.set(param_name, *value)
					.expect("Param name returned by .step() must exist.");
			}
			param_ctx.apply(&self.params, &mut self.node_comps);

			transform_ctx.update(&self.nodes, &mut self.node_comps);
		}

		if let Some(render_ctx) = self.render_ctx.as_mut() {
			render_ctx.update(&self.nodes, &mut self.node_comps);
		}
	}
}
