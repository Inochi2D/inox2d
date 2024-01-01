use std::collections::HashMap;
use std::fmt::Display;

use indextree::{Arena, NodeId};

use super::components::Composite;
use super::node::{InoxNode, InoxNodeUuid};

#[derive(Debug)]
pub struct InoxNodeTree {
	pub root: indextree::NodeId,
	pub arena: Arena<InoxNode>,
	pub uuids: HashMap<InoxNodeUuid, indextree::NodeId>,
}

impl InoxNodeTree {
	fn get_internal_node(&self, uuid: InoxNodeUuid) -> Option<&indextree::Node<InoxNode>> {
		self.arena.get(*self.uuids.get(&uuid)?)
	}
	fn get_internal_node_mut(&mut self, uuid: InoxNodeUuid) -> Option<&mut indextree::Node<InoxNode>> {
		self.arena.get_mut(*self.uuids.get(&uuid)?)
	}

	#[allow(clippy::borrowed_box)]
	pub fn get_node(&self, uuid: InoxNodeUuid) -> Option<&InoxNode> {
		Some(self.get_internal_node(uuid)?.get())
	}

	pub fn get_node_mut(&mut self, uuid: InoxNodeUuid) -> Option<&mut InoxNode> {
		Some(self.get_internal_node_mut(uuid)?.get_mut())
	}

	#[allow(clippy::borrowed_box)]
	pub fn get_parent(&self, uuid: InoxNodeUuid) -> Option<&InoxNode> {
		let node = self.get_internal_node(uuid)?;
		Some(self.arena.get(node.parent()?)?.get())
	}

	pub fn children_uuids(&self, uuid: InoxNodeUuid) -> Option<Vec<InoxNodeUuid>> {
		let node = self.get_internal_node(uuid)?;
		let node_id = self.arena.get_node_id(node)?;
		Some(
			node_id
				.children(&self.arena)
				.filter_map(|nid| self.arena.get(nid))
				.map(|nod| nod.get().uuid)
				.collect::<Vec<_>>(),
		)
	}

	fn rec_all_childen_from_node(
		&self,
		node: &InoxNode,
		zsort: f32,
		skip_composites: bool,
	) -> Vec<(InoxNodeUuid, f32)> {
		let node_state = node;
		let zsort = zsort + node_state.zsort;
		let mut vec = vec![(node_state.uuid, zsort)];

		// Skip composite children because they're a special case
		if !skip_composites || !node.components.has::<Composite>() {
			for child_uuid in self.children_uuids(node.uuid).unwrap_or_default() {
				if let Some(child) = self.get_node(child_uuid) {
					vec.extend(self.rec_all_childen_from_node(child, zsort, skip_composites));
				}
			}
		}

		vec
	}

	pub fn ancestors(&self, uuid: InoxNodeUuid) -> indextree::Ancestors<InoxNode> {
		self.uuids[&uuid].ancestors(&self.arena)
	}

	fn sort_by_zsort(&self, node: &InoxNode, skip_composites: bool) -> Vec<InoxNodeUuid> {
		let uuid_zsorts = self.rec_all_childen_from_node(node, 0.0, skip_composites);
		sort_uuids_by_zsort(uuid_zsorts)
	}

	/// all nodes, zsorted, with composite children excluded
	pub fn zsorted_root(&self) -> Vec<InoxNodeUuid> {
		let root = self.arena.get(self.root).unwrap().get();
		self.sort_by_zsort(root, true)
	}

	/// all children, grandchildren..., zsorted, with parent excluded
	pub fn zsorted_children(&self, id: InoxNodeUuid) -> Vec<InoxNodeUuid> {
		let node = self.arena.get(self.uuids[&id]).unwrap().get();
		self.sort_by_zsort(node, false)
			.into_iter()
			.filter(|uuid| *uuid != node.uuid)
			.collect::<Vec<_>>()
	}

	/// all nodes
	pub fn all_node_ids(&self) -> Vec<InoxNodeUuid> {
		self.arena.iter().map(|n| n.get().uuid).collect()
	}
}

fn rec_fmt(
	indent: usize,
	f: &mut std::fmt::Formatter<'_>,
	node_id: NodeId,
	arena: &Arena<InoxNode>,
) -> std::fmt::Result {
	let Some(node) = arena.get(node_id) else {
		return Ok(());
	};

	let node = node.get();

	let type_name = &node.type_name;
	#[cfg(feature = "owo")]
	let type_name = {
		use owo_colors::OwoColorize;
		type_name.magenta()
	};

	writeln!(f, "{}- [{}] {}", "  ".repeat(indent), type_name, node.name)?;
	for child in node_id.children(arena) {
		rec_fmt(indent + 1, f, child, arena)?;
	}

	Ok(())
}

impl Display for InoxNodeTree {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Some(root_node) = self.arena.get(self.root) else {
			return write!(f, "(empty)");
		};

		let root_node = root_node.get();

		let type_name = &root_node.type_name;
		#[cfg(feature = "owo")]
		let type_name = {
			use owo_colors::OwoColorize;
			type_name.magenta()
		};

		writeln!(f, "- [{}] {}", type_name, root_node.name)?;
		for child in self.root.children(&self.arena) {
			rec_fmt(1, f, child, &self.arena)?;
		}

		Ok(())
	}
}

fn sort_uuids_by_zsort(mut uuid_zsorts: Vec<(InoxNodeUuid, f32)>) -> Vec<InoxNodeUuid> {
	uuid_zsorts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
	uuid_zsorts.into_iter().map(|(uuid, _zsort)| uuid).collect()
}
