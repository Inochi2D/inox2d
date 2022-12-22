use std::{any::TypeId, collections::HashMap};

use crate::nodes::{node::Node, node_tree::NodeTree};

use self::vbo::Vbo;

pub mod app;
pub mod vbo;
pub mod texture;
pub struct WgpuRenderer {
    pub nodes: NodeTree,
    pub vao: glow::NativeVertexArray,
    pub verts: Vbo<f32>,
    pub uvs: Vbo<f32>,
    pub deform: Vbo<f32>,
    pub ibo: Vbo<u16>,
    pub textures: Vec<glow::NativeTexture>,
    pub node_renderers: HashMap<TypeId, ErasedNodeRenderer>,
}

type ErasedNodeRenderer = Box<dyn Fn(&WgpuRenderer, &dyn Node)>;
