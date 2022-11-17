use crate::math::transform::Transform;

use super::puppet::PuppetId;

pub mod drawable;
pub mod part;

pub(crate) fn in_init_nodes() {
    todo!();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeId(usize);

#[derive(Clone, Debug)]
pub struct NodeBaseState {
    id: NodeId,
    uuid: u32,
    path: String,
    puppet: PuppetId,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    zsort: f32,
    /// The offset to apply to sorting.
    offset_sort: f32,
    lock_to_root: bool,
    /// The cached world space transform of the node.
    global_transform: Transform,
    /// The local transform of the node.
    local_transform: Transform,
    /// The offset to the transform to apply.
    offset_transform: Transform,
    recalculate_transform: bool,
    /// Whether the node is enabled.
    enabled: bool,
    /// Visual name of the node.
    name: String,
    
}

pub trait Node {
    const TYPE_ID: &'static str;


}