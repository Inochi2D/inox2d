use serde::{Deserialize, Serialize, Deserializer};

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

#[typetag::serde]
impl Node for Part {
    fn get_node_state(&self) -> &NodeState {
        &self.node_state
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        &mut self.node_state
    }
}

impl<'de, D> NodeDeserializer<'de, D> for Part
where
    D: Deserializer<'de>,
{
    const NODE_TYPE: &'static str = "Part";

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node>, D::Error> {
        let part: Self = Self::deserialize(deserializer)?;
        Ok(Box::new(part))
    }
}
