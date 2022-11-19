use serde::{Serialize, Deserialize, Serializer, Deserializer};

use super::drawable::Drawable;
use super::node::{NodeState, Node, NodeDeserializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    #[serde(flatten)]
    node_state: NodeState,
    #[serde(flatten)]
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

impl<'de, D, S> NodeDeserializer<'de, D, S> for Composite
where
    D: Deserializer<'de>,
    S: Serializer,
{
    const NODE_TYPE: &'static str = "Composite";

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node<S>>, D::Error> {
        let part: Self = Self::deserialize(deserializer)?;
        Ok(Box::new(part))
    }
}
