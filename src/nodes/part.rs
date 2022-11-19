use serde::{Deserialize, Serialize, Serializer, Deserializer};

use crate::mesh::Mesh;

use super::drawable::Drawable;
use super::node::{Node, NodeState, NodeDeserializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(flatten)]
    node_state: NodeState,
    #[serde(flatten)]
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

impl<'de, D, S> NodeDeserializer<'de, D, S> for Part
where
    D: Deserializer<'de>,
    S: Serializer,
{
    const NODE_TYPE: &'static str = "Part";

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node<S>>, D::Error> {
        let part: Self = Self::deserialize(deserializer)?;
        Ok(Box::new(part))
    }
}
