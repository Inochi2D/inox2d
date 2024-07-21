use crate::node::components::TransformStore;

use super::Puppet;

impl Puppet {
	/// Give every node a `TransformStore` component, if the puppet is going to be rendered/animated
	pub(super) fn init_node_transforms(&mut self) {
		for node in self.nodes.iter() {
			self.node_comps.add(node.uuid, TransformStore::default());
		}
	}

	/// Update the puppet's nodes' absolute transforms, by combining transforms
	/// from each node's ancestors in a pre-order traversal manner.
	pub(crate) fn update_trans(&mut self) {
		let root_trans_store = self
			.node_comps
			.get_mut::<TransformStore>(self.nodes.root_node_id)
			.unwrap();
		// The root's absolute transform is its relative transform.
		let root_trans = root_trans_store.relative.to_matrix();
		root_trans_store.absolute = root_trans;

		// Pre-order traversal, just the order to ensure that parents are accessed earlier than children
		// Skip the root
		for node in self.nodes.pre_order_iter().skip(1) {
			let base_trans = if node.lock_to_root {
				root_trans
			} else {
				let parent = self.nodes.get_parent(node.uuid);
				self.node_comps.get_mut::<TransformStore>(parent.uuid).unwrap().absolute
			};

			let node_trans_store = self.node_comps.get_mut::<TransformStore>(node.uuid).unwrap();
			let node_trans = node_trans_store.relative.to_matrix();
			node_trans_store.absolute = base_trans * node_trans;
		}
	}
}
