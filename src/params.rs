use glam::Vec2;

use crate::math::interp::InterpolateMode;
use crate::math::matrix::Matrix2d;
use crate::nodes::node::InoxNodeUuid;

/// Parameter binding to a node. This allows to animate a node based on the value of the parameter that owns it.
#[derive(Debug)]
pub struct Binding {
    pub node: InoxNodeUuid,
    pub is_set: Matrix2d<bool>,
    pub interpolate_mode: InterpolateMode,
    pub values: BindingValues,
}

#[derive(Debug)]
pub enum BindingValues {
    ZSort(Matrix2d<f32>),
    TransformTX(Matrix2d<f32>),
    TransformTY(Matrix2d<f32>),
    TransformSX(Matrix2d<f32>),
    TransformSY(Matrix2d<f32>),
    TransformRX(Matrix2d<f32>),
    TransformRY(Matrix2d<f32>),
    TransformRZ(Matrix2d<f32>),
    Deform(Matrix2d<Vec<Vec2>>),
}

/// Parameter. A simple bounded value that is used to animate nodes through bindings.
#[derive(Debug)]
pub struct Param {
    pub uuid: u32,
    pub name: String,
    pub is_vec2: bool,
    pub min: Vec2,
    pub max: Vec2,
    pub defaults: Vec2,
    pub axis_points: [Vec<f32>; 2],
    pub bindings: Vec<Binding>,
}
