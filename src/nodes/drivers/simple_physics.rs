use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::impl_node;
use crate::nodes::node::NodeState;

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

impl_node!(SimplePhysics, node_state);