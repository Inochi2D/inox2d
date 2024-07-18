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
	physics_ctx: Option<Box<Vec<InoxNodeUuid>>>,
	pub nodes: InoxNodeTree,
	pub node_comps: World,
	render_ctx: Option<RenderCtx>,
	pub(crate) params: HashMap<ParamUuid, Param>,
	pub(crate) param_names: HashMap<String, ParamUuid>,
}

impl Puppet {
	pub fn new(meta: PuppetMeta, physics: PuppetPhysics, root: InoxNode, named_params: HashMap<String, Param>) -> Self {
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
}

/// Global physics parameters for the puppet.
pub struct PuppetPhysics {
	pub pixels_per_meter: f32,
	pub gravity: f32,
}
