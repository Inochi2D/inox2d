pub mod meta;
mod transforms;
mod tree;
mod world;

use std::collections::HashMap;

use crate::node::{InoxNode, InoxNodeUuid};
use crate::params::{Param, ParamCtx};
use crate::render::RenderCtx;

use meta::PuppetMeta;
pub use tree::InoxNodeTree;
pub use world::World;

/// Inochi2D puppet.
pub struct Puppet {
	pub meta: PuppetMeta,
	physics: PuppetPhysics,
	// TODO: define the actual ctx
	pub(crate) nodes: InoxNodeTree,
	pub(crate) node_comps: World,
	/// Context for rendering this puppet. See `.init_render_ctx()`.
	pub render_ctx: Option<RenderCtx>,
	pub(crate) params: HashMap<String, Param>,
	pub(crate) param_ctx: Option<ParamCtx>,
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
			nodes: InoxNodeTree::new_with_root(root),
			node_comps: World::new(),
			render_ctx: None,
			params,
			param_ctx: None,
		}
	}

	/// Call this on a freshly loaded puppet if rendering is needed. Panicks on second call.
	pub fn init_rendering(&mut self) {
		if self.render_ctx.is_some() {
			panic!("Puppet already initialized for rendering.");
		}

		self.init_node_transforms();

		let render_ctx = RenderCtx::new(self);
		self.render_ctx = Some(render_ctx);
	}

	/// Call this on a puppet if params are going to be used. Panicks on second call.
	pub fn init_params(&mut self) {
		if self.param_ctx.is_some() {
			panic!("Puppet already initialized for params.");
		}

		let param_ctx = ParamCtx::new(self);
		self.param_ctx = Some(param_ctx);
	}
}

/// Global physics parameters for the puppet.
pub struct PuppetPhysics {
	pub pixels_per_meter: f32,
	pub gravity: f32,
}
