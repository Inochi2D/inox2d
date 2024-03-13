use std::collections::HashMap;

use indextree::Arena;

use crate::node::{InoxNode, InoxNodeUuid};

pub struct InoxNodeTree {
	root_id: indextree::NodeId,
	arena: Arena<InoxNode>,
	node_ids: HashMap<InoxNodeUuid, indextree::NodeId>,
}

impl InoxNodeTree {
	pub fn new_with_root(node: InoxNode) -> Self {
		let id = node.uuid;
		let mut node_ids = HashMap::new();
		let mut arena = Arena::new();

		let root_id = arena.new_node(node);
		node_ids.insert(id, root_id);

		Self {
			root_id,
			arena,
			node_ids,
		}
	}

	pub fn add(&mut self, parent: InoxNodeUuid, id: InoxNodeUuid, node: InoxNode) {
		let parent_id = self.node_ids.get(&parent).expect("parent should be added earlier");

		let node_id = self.arena.new_node(node);
		parent_id.append(node_id, &mut self.arena);

		let result = self.node_ids.insert(id, node_id);
		if result.is_some() {
			panic!("duplicate inox node uuid")
		}
	}

	fn get_internal_node(&self, id: InoxNodeUuid) -> Option<&indextree::Node<InoxNode>> {
		self.arena.get(*self.node_ids.get(&id)?)
	}

	fn get_internal_node_mut(&mut self, id: InoxNodeUuid) -> Option<&mut indextree::Node<InoxNode>> {
		self.arena.get_mut(*self.node_ids.get(&id)?)
	}

	pub fn get_node(&self, id: InoxNodeUuid) -> Option<&InoxNode> {
		Some(self.get_internal_node(id)?.get())
	}

	pub fn get_node_mut(&mut self, id: InoxNodeUuid) -> Option<&mut InoxNode> {
		Some(self.get_internal_node_mut(id)?.get_mut())
	}
}
