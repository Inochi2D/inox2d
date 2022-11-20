use std::fmt::Debug;

// use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};

use crate::math::transform::Transform;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct NodeUuid(pub(crate) u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    // #[serde(skip)]
    // id: NodeId,
    // #[serde(skip)]
    // parent: Option<NodeId>,
    // #[serde(skip)]
    // children: Vec<NodeId>,
    uuid: NodeUuid,
    name: String,
    enabled: bool,
    zsort: f32,
    transform: Transform,
    #[serde(rename = "lockToRoot")]
    lock_to_root: bool,
}

// TODO: make a derive macro for this
#[typetag::serde(tag = "type")]
pub trait Node: Debug {
    fn get_node_state(&self) -> &NodeState;
    fn get_node_state_mut(&mut self) -> &mut NodeState;
}

// TODO: make a derive macro for this
pub trait NodeDeserializer<'de, D: Deserializer<'de>> {
    const NODE_TYPE: &'static str;

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node>, D::Error>;
}

#[typetag::serde(name = "Node")]
impl Node for NodeState {
    fn get_node_state(&self) -> &NodeState {
        self
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        self
    }
}

// impl<'de, D> NodeDeserializer<'de, D> for NodeState
// where
//     D: Deserializer<'de>,
// {
//     const NODE_TYPE: &'static str = "Node";

//     fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node>, D::Error> {
//         let part: Self = Self::deserialize(deserializer)?;
//         Ok(Box::new(part))
//     }
// }