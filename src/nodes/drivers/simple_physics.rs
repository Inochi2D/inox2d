use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::nodes::node::{Node, NodeState};

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

#[typetag::serde]
impl Node for SimplePhysics {
    fn get_node_state(&self) -> &NodeState {
        &self.node_state
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        &mut self.node_state
    }
}
