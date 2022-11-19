use serde::{Serialize, Deserialize, Serializer};

use crate::mesh::Mesh;

use super::drawable::Drawable;
use super::node::{NodeState, Node};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    node_state: NodeState,
    draw_state: Drawable,
    mesh: Mesh,
    textures: Vec<u32>,
}

impl<S: Serializer> Node<S> for Part {
    fn get_node_state(&self) -> &NodeState {
        &self.node_state
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        &mut self.node_state
    }

    fn serialize_node(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.serialize(serializer)
    }
}