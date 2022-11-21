use std::collections::BTreeMap;

use indextree::Arena;
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
