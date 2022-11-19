use std::fmt::Debug;

use serde::{Serialize, Serializer, Deserialize, Deserializer};

use crate::math::transform::Transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    id: NodeId,
    node_type: String,
    #[serde(default)]
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    uuid: u32,
    name: String,
    enabled: bool,
    zsort: f32,
    transform: Transform,
    lock_to_root: bool,
}

// TODO: make a derive macro for this
pub trait Node<S: Serializer>: Debug {
    fn get_node_state(&self) -> &NodeState;
    fn get_node_state_mut(&mut self) -> &mut NodeState;
    fn serialize_node(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

// TODO: make a derive macro for this
pub trait NodeDeserializer<'de, D: Deserializer<'de>, S: Serializer> {
    const NODE_TYPE: &'static str;

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node<S>>, D::Error>;
}
