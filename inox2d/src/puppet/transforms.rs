use crate::node::components::{TransformStore, ZSort};

use super::{InoxNodeTree, Puppet, World};

pub(crate) struct TransformCtx {}

impl TransformCtx {
	/// Give every node a `TransformStore` and a `ZSort` component, if the puppet is going to be rendered/animated
	pub fn new(puppet: &mut Puppet) -> Self {
		for node in puppet.nodes.iter() {
			puppet.node_comps.add(node.uuid, TransformStore::default());
			puppet.node_comps.add(node.uuid, ZSort::default());
		}
		TransformCtx {}
	}

	/// Reset all transform/zsort values to default.
	pub fn reset(&mut self, nodes: &InoxNodeTree, comps: &mut World) {
		for node in nodes.iter() {
			comps.get_mut::<TransformStore>(node.uuid).unwrap().relative = node.trans_offset;
		}
	}

	/// Update the puppet's nodes' absolute transforms, by combining transforms
	/// from each node's ancestors in a pre-order traversal manner.
	pub(crate) fn update(&mut self, nodes: &InoxNodeTree, comps: &mut World) {
		let root_trans_store = comps.get_mut::<TransformStore>(nodes.root_node_id).unwrap();
		// The root's absolute transform is its relative transform.
		let root_trans = root_trans_store.relative.to_matrix();
		root_trans_store.absolute = root_trans;

		// Pre-order traversal, just the order to ensure that parents are accessed earlier than children
		// Skip the root
		for node in nodes.pre_order_iter().skip(1) {
			let base_trans = if node.lock_to_root {
				root_trans
			} else {
				let parent = nodes.get_parent(node.uuid);
				comps.get_mut::<TransformStore>(parent.uuid).unwrap().absolute
			};

			let node_trans_store = comps.get_mut::<TransformStore>(node.uuid).unwrap();
			let node_trans = node_trans_store.relative.to_matrix();
			node_trans_store.absolute = base_trans * node_trans;
		}
	}
}
