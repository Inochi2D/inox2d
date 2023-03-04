use std::borrow::Borrow;

use glam::Vec2;

use crate::{
    nodes::node_tree::InoxNodeTree,
    puppet::{Binding, Param, PuppetPhysics},
};

struct BakedParam {
    pub uuid: u32,
    pub name: String,
    pub is_vec2: bool,
    pub min: Vec2,
    pub max: Vec2,
    pub defaults: Vec2,
    pub axis_points: [Vec<f32>; 2],
    pub bindings: Vec<Binding>,
}

pub struct BakedPuppet {
    physics: PuppetPhysics,
    parameters: Vec<BakedParam>,
    nodes: InoxNodeTree,

    // This needs to handle buffer management now (so probably BakedModel)
    verts: Vec<Vec2>,
    deforms: Vec<Vec2>,
    uvs: Vec<Vec2>,
    // uvs don't change so we can let the renderer allocate that?
    // or do we just want to do it here
}

impl BakedPuppet {
    // TODO: From<Puppet> / From<Model>

    // This is an expensive operation. Try to minimize it.
    pub fn update_parameters<S: AsRef<str>, I: Iterator<Item = (S, Vec2)>>(&mut self, iter: I) {}

    // all the stuff that needs to be exposed to C goes here
    // so verts, textures (?) (do I make this a BakedModel instead of BakedPuppet),
    // blah blah
}
