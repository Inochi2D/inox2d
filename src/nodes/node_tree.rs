use std::any::TypeId;
use std::collections::BTreeMap;
// use std::fmt::Display;

use indextree::Arena;

use super::composite::Composite;
use super::node::{Node, NodeUuid};

#[derive(Debug)]
pub struct NodeTree {
    pub root: indextree::NodeId,
    pub arena: Arena<Box<dyn Node>>,
    pub uuids: BTreeMap<NodeUuid, indextree::NodeId>,
}

impl NodeTree {
    fn get_internal_node(&self, uuid: NodeUuid) -> Option<&indextree::Node<Box<dyn Node>>> {
        self.arena.get(*self.uuids.get(&uuid)?)
    }
    fn get_internal_node_mut(
        &mut self,
        uuid: NodeUuid,
    ) -> Option<&mut indextree::Node<Box<dyn Node>>> {
        self.arena.get_mut(*self.uuids.get(&uuid)?)
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_node(&self, uuid: NodeUuid) -> Option<&Box<dyn Node>> {
        Some(self.get_internal_node(uuid)?.get())
    }

    pub fn get_node_mut(&mut self, uuid: NodeUuid) -> Option<&mut Box<dyn Node>> {
        Some(self.get_internal_node_mut(uuid)?.get_mut())
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_parent(&self, uuid: NodeUuid) -> Option<&Box<dyn Node>> {
        let node = self.get_internal_node(uuid)?;
        Some(self.arena.get(node.parent()?)?.get())
    }

    pub fn get_children_uuids(&self, uuid: NodeUuid) -> Option<Vec<NodeUuid>> {
        let node = self.get_internal_node(uuid)?;
        let node_id = self.arena.get_node_id(node)?;
        Some(
            node_id
                .children(&self.arena)
                .filter_map(|nid| self.arena.get(nid))
                .map(|nod| nod.get().get_node_state().uuid)
                .collect::<Vec<_>>(),
        )
    }

    pub fn ancestors(&self, uuid: NodeUuid) -> indextree::Ancestors<Box<dyn Node>> {
        self.uuids[&uuid].ancestors(&self.arena)
    }

    fn rec_zsorts_from_root(&self, node: &dyn Node, zsort: f32) -> Vec<(NodeUuid, f32)> {
        let node_state = node.get_node_state();
        let zsort = zsort + node_state.zsort;
        let mut vec = vec![(node_state.uuid, zsort)];

        // Skip composite children because they're a special case
        if node.type_id() != TypeId::of::<Composite>() {
            for child_uuid in self.get_children_uuids(node_state.uuid).unwrap_or_default() {
                if let Some(child) = self.get_node(child_uuid) {
                    vec.extend(self.rec_zsorts_from_root(child.as_ref(), zsort));
                }
            }
        }

        vec
    }

    fn sort_uuids_by_zsort(&self, mut uuid_zsorts: Vec<(NodeUuid, f32)>) -> Vec<NodeUuid> {
        uuid_zsorts.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().reverse());
        uuid_zsorts.into_iter().map(|(uuid, _zsort)| uuid).collect()
    }

    fn sort_by_zsort(&self, node: &dyn Node) -> Vec<NodeUuid> {
        let uuid_zsorts = self.rec_zsorts_from_root(node, 0.);
        self.sort_uuids_by_zsort(uuid_zsorts)
    }

    pub fn zsorted(&self) -> Vec<NodeUuid> {
        let root = self.arena.get(self.root).unwrap().get();
        self.sort_by_zsort(root.as_ref())
    }
}

// fn rec_fmt(
//     indent: usize,
//     f: &mut std::fmt::Formatter<'_>,
//     node_id: NodeId,
//     arena: &Arena<Box<dyn Node>>,
// ) -> std::fmt::Result {
//     let Some(node) = arena.get(node_id) else {
//         return Ok(());
//     };

//     let node = node.get();

//     let type_name = node.typetag_name();
//     #[cfg(feature = "owo")]
//     let type_name = {
//         use owo_colors::OwoColorize;
//         type_name.magenta()
//     };

//     writeln!(
//         f,
//         "{}- [{}] {}",
//         "  ".repeat(indent),
//         type_name,
//         node.get_node_state().name
//     )?;
//     for child in node_id.children(arena) {
//         rec_fmt(indent + 1, f, child, arena)?;
//     }

//     Ok(())
// }

// impl Display for NodeTree {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let Some(root_node) = self.arena.get(self.root) else {
//             return write!(f, "(empty)");
//         };

//         let root_node = root_node.get();

//         let type_name = root_node.typetag_name();
//         #[cfg(feature = "owo")]
//         let type_name = {
//             use owo_colors::OwoColorize;
//             type_name.magenta()
//         };

//         writeln!(f, "- [{}] {}", type_name, root_node.get_node_state().name)?;
//         for child in self.root.children(&self.arena) {
//             rec_fmt(1, f, child, &self.arena)?;
//         }

//         Ok(())
//     }
// }
