use glam::Vec2;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

use crate::nodes::node::{NodeState, Node, NodeDeserializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePhysics {
    #[serde(flatten)]
    node_state: NodeState,
    param: u32,
    model_type: String,
    map_mode: String,
    gravity: f32,
    length: f32,
    frequency: f32,
    angle_damping: f32,
    length_damping: f32,
    output_scale: Vec2,
}

impl<S: Serializer> Node<S> for SimplePhysics {
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

impl<'de, D, S> NodeDeserializer<'de, D, S> for SimplePhysics
where
    D: Deserializer<'de>,
    S: Serializer,
{
    const NODE_TYPE: &'static str = "SimplePhysics";

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node<S>>, D::Error> {
        let part: Self = Self::deserialize(deserializer)?;
        Ok(Box::new(part))
    }
}
