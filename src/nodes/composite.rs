use serde::{Serialize, Deserialize, Deserializer};

use super::drawable::Drawable;
use super::node::{NodeState, Node, NodeDeserializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    #[serde(flatten)]
    node_state: NodeState,
    #[serde(flatten)]
    draw_state: Drawable,
}

impl Node for Composite {
    fn get_node_state(&self) -> &NodeState {
        &self.node_state
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        &mut self.node_state
    }
}

impl<'de, D> NodeDeserializer<'de, D> for Composite
where
    D: Deserializer<'de>,
{
    const NODE_TYPE: &'static str = "Composite";

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node>, D::Error> {
        let part: Self = Self::deserialize(deserializer)?;
        Ok(Box::new(part))
    }
}
