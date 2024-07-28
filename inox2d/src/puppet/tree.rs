use std::collections::HashMap;

use indextree::Arena;

use crate::node::{InoxNode, InoxNodeUuid};

pub struct InoxNodeTree {
	// make this public, instead of replicating all node methods for root, now that callers have the root id
	pub root_node_id: InoxNodeUuid,
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
			root_node_id: id,
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

	/// order is not guaranteed. use pre_order_iter() for pre-order traversal
	pub fn iter(&self) -> impl Iterator<Item = &InoxNode> {
		self.arena.iter().map(|n| {
			if n.is_removed() {
				panic!("There is a removed node inside the indextree::Arena of the node tree.")
			}
			n.get()
		})
	}

	pub fn pre_order_iter(&self) -> impl Iterator<Item = &InoxNode> {
		let root_id = self.node_ids.get(&self.root_node_id).unwrap();
		root_id
			.descendants(&self.arena)
			.map(|id| self.arena.get(id).unwrap().get())
	}

	/// WARNING: panicks if called on root
	pub fn get_parent(&self, children: InoxNodeUuid) -> &InoxNode {
		self.arena
			.get(
				self.arena
					.get(*self.node_ids.get(&children).unwrap())
					.unwrap()
					.parent()
					.unwrap(),
			)
			.unwrap()
			.get()
	}

	pub fn get_children(&self, parent: InoxNodeUuid) -> impl Iterator<Item = &InoxNode> {
		self.node_ids
			.get(&parent)
			.unwrap()
			.children(&self.arena)
			.map(|id| self.arena.get(id).unwrap().get())
	}
}
