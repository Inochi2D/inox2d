use std::borrow::Borrow;
use std::collections::BTreeMap;

use self::node::{Node, NodeUuid};

pub mod node;
pub mod drivers;

pub mod composite;
pub mod drawable;
pub mod part;

#[derive(Debug, Default)]
pub struct NodeTree {
    arena: BTreeMap<NodeUuid, Box<dyn Node>>
}

impl NodeTree {
    pub fn insert(&mut self, node: Box<dyn Node>) -> NodeUuid {
        let node_id: NodeUuid = NodeUuid(self.arena.len() as u32);
        self.arena.insert(node_id, node);
        node_id
    }

    pub fn get_node(&self, node_id: NodeUuid) -> Option<&dyn Node> {
        self.arena.get(&node_id).map(Box::borrow)
    }

    pub fn get_node_mut(&mut self, node_id: NodeUuid) -> Option<&mut Box<dyn Node>> {
        self.arena.get_mut(&node_id)
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }
}