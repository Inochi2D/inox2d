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
}

// This is the most atrocious function I have ever written 💀
// TODO: replace with trait upcasting, coming in future Rust releases
#[cfg(feature = "opengl")]
pub(crate) fn downcast_node<'a, N: Node + 'static>(node: &'a dyn Node) -> Option<&'a N> {
    if node.type_id() == std::any::TypeId::of::<N>() {
        // downcasting magic <_<
        // SAFETY: we just checked that the node is of type N
        let node = unsafe { &*(node as *const dyn Node as *const N) };
        Some(node)
    } else {
        None
    }
}

#[cfg(feature = "opengl")]
pub(crate) fn downcast_node_mut<'a, N: Node + 'static>(node: &'a mut Box<dyn Node>) -> Option<&'a mut N> {
    let type_id = node.as_ref().type_id();
    if type_id == std::any::TypeId::of::<N>() {
        // downcasting magic <_<
        // SAFETY: we just checked that the node is of type N
        let node = unsafe { &mut *(node.as_mut() as *mut dyn Node as *mut N) };
        Some(node)
    } else {
        None
    }
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
