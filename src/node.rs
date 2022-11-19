#![allow(unused)]

use std::any::Any;
use std::fmt::Debug;

use glam::{Vec3, Vec2};
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use crate::math::transform::Transform;
use crate::mesh::Mesh;

/// Blending modes
#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Normal blending mode.
    Normal,
    /// Multiply blending mode.
    Multiply,
    /// Color Dodge.
    ColorDodge,
    /// Linear Dodge.
    LinearDodge,
    /// Screen.
    Screen,
    /// Clip to Lower.
    /// Special blending mode that clips the drawable
    /// to a lower rendered area.
    ClipToLower,
    /// Slice from Lower.
    /// Special blending mode that slices the drawable
    /// via a lower rendered area.
    /// (Basically inverse ClipToLower.)
    SliceFromLower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(usize);

impl NodeId {
    /// Creates a new `NodeId`.
    /// 
    /// # Safety
    /// 
    /// This function should never be called manually, since it may point to a node that does not exist.
    pub unsafe fn new(value: usize) -> Self {
        NodeId(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    id: NodeId,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    uuid: u32,
    name: String,
    enabled: bool,
    zsort: f32,
    transform: Transform,
    lock_to_root: bool,
}

pub trait Node<S: Serializer>: Debug {
    fn node_state(&self) -> &NodeState;
    fn node_state_mut(&mut self) -> &mut NodeState;
    fn serialize_node(&self, serializer: S);
}

pub trait NodeDeserializer<'de, D: Deserializer<'de>, S: Serializer> {
    const NODE_TYPE: &'static str;

    fn deserialize_node(&self, deserializer: D) -> Box<dyn Node<S>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawState {
    blend_mode: BlendMode,
    tint: Vec3,
    screen_tint: Vec3,
    mask_threshold: f32,
    opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    node_state: NodeState,
    draw_state: DrawState,
    mesh: Mesh,
    textures: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    node_state: NodeState,
    draw_state: DrawState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePhysics {
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

#[derive(Debug, Default)]
pub struct NodeTree<S: Serialize>(Vec<Box<dyn Node<S>>>);
