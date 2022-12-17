use std::any::Any;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::math::transform::Transform;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[repr(transparent)]
pub struct NodeUuid(pub(crate) u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub uuid: NodeUuid,
    pub name: String,
    pub enabled: bool,
    pub zsort: f32,
    pub transform: Transform,
    #[serde(rename = "lockToRoot")]
    pub lock_to_root: bool,
}

// TODO: make a derive macro for this
#[typetag::serde(tag = "type")]
pub trait Node: Debug + Any {
    fn get_node_state(&self) -> &NodeState;
    fn get_node_state_mut(&mut self) -> &mut NodeState;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[typetag::serde(name = "Node")]
impl Node for NodeState {
    fn get_node_state(&self) -> &NodeState {
        self
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
