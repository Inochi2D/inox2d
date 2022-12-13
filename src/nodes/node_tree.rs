use std::collections::BTreeMap;
use std::fmt::Display;

use indextree::{Arena, NodeId};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use super::node::{Node, NodeUuid};

/// Node tree struct who's only purpose is to be deserialized into an arena.
#[derive(Debug, Serialize, Deserialize)]
struct SNodeTree {
    #[serde(flatten)]
    node: Box<dyn Node>,
    #[serde(default)]
    children: Vec<SNodeTree>,
}

impl SNodeTree {
    /// Moves the entire tree into an arena.
    /// Returns the Node ID of the root of the tree.
    fn flatten_into(
        self,
        arena: &mut Arena<Box<dyn Node>>,
        uuids: &mut BTreeMap<NodeUuid, indextree::NodeId>,
    ) -> indextree::NodeId {
        let uuid = self.node.get_node_state().uuid;
        let node_id = arena.new_node(self.node);
        uuids.insert(uuid, node_id);
        for child in self.children {
            child.flatten_children(arena, node_id, uuids);
        }
        node_id
    }

    fn flatten_children(
        self,
        arena: &mut Arena<Box<dyn Node>>,
        parent: indextree::NodeId,
        uuids: &mut BTreeMap<NodeUuid, indextree::NodeId>,
    ) {
        let uuid = self.node.get_node_state().uuid;
        let node_id = arena.new_node(self.node);
        uuids.insert(uuid, node_id);
        parent.append(node_id, arena);
        for child in self.children {
            child.flatten_children(arena, node_id, uuids);
        }
    }
}

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
}

fn rec_fmt(
    indent: usize,
    f: &mut std::fmt::Formatter<'_>,
    node_id: NodeId,
    arena: &Arena<Box<dyn Node>>,
) -> std::fmt::Result {
    let Some(node) = arena.get(node_id) else {
        return Ok(());
    };

    let node = node.get();
    writeln!(
        f,
        "{}- [{}] {}",
        "  ".repeat(indent),
        node.typetag_name().magenta(),
        node.get_node_state().name
    )?;
    for child in node_id.children(arena) {
        rec_fmt(indent + 1, f, child, arena)?;
    }

    Ok(())
}

impl Display for NodeTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(root_node) = self.arena.get(self.root) else {
            return write!(f, "(empty)");
        };

        let root_node = root_node.get();
        writeln!(
            f,
            "- [{}] {}",
            root_node.typetag_name().magenta(),
            root_node.get_node_state().name
        )?;
        for child in self.root.children(&self.arena) {
            rec_fmt(1, f, child, &self.arena)?;
        }

        Ok(())
    }
}

impl Serialize for NodeTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_indextree::Node::new(self.root, &self.arena).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NodeTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let node_tree = SNodeTree::deserialize(deserializer)?;
        Ok(Self::from(node_tree))
    }
}

impl From<SNodeTree> for NodeTree {
    fn from(sntree: SNodeTree) -> Self {
        let mut arena = Arena::new();
        let mut uuids = BTreeMap::new();
        let root = sntree.flatten_into(&mut arena, &mut uuids);
        NodeTree { root, arena, uuids }
    }
}
