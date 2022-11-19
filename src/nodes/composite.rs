use serde::{Serialize, Deserialize, Serializer};

use super::drawable::Drawable;
use super::node::{NodeState, Node};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    node_state: NodeState,
    draw_state: Drawable,
}

impl<S: Serializer> Node<S> for Composite {
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