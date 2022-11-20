use std::collections::BTreeMap;

use indextree::Arena;
use serde::{Deserialize, Serialize};

use super::node::{Node, NodeUuid};

#[derive(Debug, Serialize, Deserialize)]
pub struct SNodeTree {
    #[serde(flatten)]
    node: Box<dyn Node>,
    #[serde(default)]
    children: Vec<SNodeTree>,
}

impl SNodeTree {
    fn flatten_into(
        self,
        arena: &mut Arena<Box<dyn Node>>,
        uuids: &mut BTreeMap<NodeUuid, indextree::NodeId>,
    ) {
        let uuid = self.node.get_node_state().uuid;
        let node_id = arena.new_node(self.node);
        uuids.insert(uuid, node_id);
        for child in self.children {
            child.flatten_children(arena, node_id, uuids);
        }
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
    arena: Arena<Box<dyn Node>>,
    uuids: BTreeMap<NodeUuid, indextree::NodeId>,
}

impl From<SNodeTree> for NodeTree {
    fn from(sntree: SNodeTree) -> Self {
        let mut arena = Arena::new();
        let mut uuids = BTreeMap::new();
        sntree.flatten_into(&mut arena, &mut uuids);
        NodeTree { arena, uuids }
    }
}
