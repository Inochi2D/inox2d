pub mod meta;
mod transforms;
mod tree;
mod world;

use std::collections::HashMap;

use crate::node::{InoxNode, InoxNodeUuid};
use crate::params::{Param, ParamUuid};
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
	pub(crate) params: HashMap<ParamUuid, Param>,
	pub(crate) param_names: HashMap<String, ParamUuid>,
}

impl Puppet {
	pub(crate) fn new(
		meta: PuppetMeta,
		physics: PuppetPhysics,
		root: InoxNode,
		named_params: HashMap<String, Param>,
	) -> Self {
		let mut params = HashMap::new();
		let mut param_names = HashMap::new();
		for (name, param) in named_params {
			param_names.insert(name, param.uuid);
			params.insert(param.uuid, param);
		}

		Self {
			meta,
			physics,
			physics_ctx: None,
			nodes: InoxNodeTree::new_with_root(root),
			node_comps: World::new(),
			render_ctx: None,
			params,
			param_names,
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
}

/// Global physics parameters for the puppet.
pub struct PuppetPhysics {
	pub pixels_per_meter: f32,
	pub gravity: f32,
}
