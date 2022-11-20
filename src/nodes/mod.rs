use std::borrow::Borrow;
use std::collections::BTreeMap;

use serde::Serialize;

use self::node::{Node, NodeId};

pub mod node;
pub mod drivers;

pub mod composite;
pub mod drawable;
pub mod part;

#[derive(Debug, Default)]
pub struct NodeTree<S: Serialize> {
    arena: BTreeMap<NodeId, Box<dyn Node<S>>>
}

impl<S: Serialize> NodeTree<S> {
    pub fn insert(&mut self, node: Box<dyn Node<S>>) -> NodeId {
        let node_id: NodeId = NodeId(self.arena.len());
        self.arena.insert(node_id, node);
        node_id
    }

    pub fn get_node(&self, node_id: NodeId) -> Option<&dyn Node<S>> {
        self.arena.get(&node_id).map(Box::borrow)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut Box<dyn Node<S>>> {
        self.arena.get_mut(&node_id)
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }
}