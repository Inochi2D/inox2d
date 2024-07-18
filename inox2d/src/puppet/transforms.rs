use crate::math::transform::Transform;

use super::Puppet;

impl Puppet {
	/// give every node a Transform component for the absolute transform, if the puppet is going to be rendered/animated
	fn init_node_transforms(&mut self) {
		for node in self.nodes.iter() {
			self.node_comps.add(node.uuid, Transform::default());
		}
	}

	/// Update the puppet's nodes' absolute transforms, by combining transforms
	/// from each node's ancestors in a pre-order traversal manner.
	pub(crate) fn update_trans(&mut self) {
		let root_node = self.nodes.get_node(self.nodes.root_node_id).unwrap();

		// The root's absolute transform is its relative transform.
		let root_trans = root_node.trans_offset.to_matrix();
		let root_trans_comp = self.node_comps.get_mut::<Transform>(root_node.uuid).unwrap();
		root_trans_comp.mat = root_trans;

		// Pre-order traversal, just the order to ensure that parents are accessed earlier than children
		// Skip the root
		for node in self.nodes.pre_order_iter().skip(1) {
			let base_trans = if node.lock_to_root {
				root_trans
			} else {
				let parent = self.nodes.get_parent(node.uuid);
				self.node_comps.get_mut::<Transform>(parent.uuid).unwrap().mat
			};

			let node_trans_comp = self.node_comps.get_mut::<Transform>(node.uuid).unwrap();
			node_trans_comp.mat = base_trans * node.trans_offset.to_matrix();
		}
	}
}
