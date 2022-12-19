use std::any::Any;
use std::fmt::Debug;

use crate::math::transform::Transform;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[repr(transparent)]
pub struct NodeUuid(pub(crate) u32);

#[derive(Debug, Clone)]
pub struct NodeState {
    pub uuid: NodeUuid,
    pub name: String,
    pub enabled: bool,
    pub zsort: f32,
    pub transform: Transform,
    pub lock_to_root: bool,
}

pub trait Node: Debug + Any {
    fn get_node_state(&self) -> &NodeState;
    fn get_node_state_mut(&mut self) -> &mut NodeState;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
