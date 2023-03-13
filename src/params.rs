use glam::Vec2;

use crate::nodes::node::InoxNodeUuid;

#[derive(Debug)]
pub enum InterpolateMode {
    Linear,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown interpolate mode {0:?}")]
pub struct UnknownInterpolateModeError(String);

impl TryFrom<&str> for InterpolateMode {
    type Error = UnknownInterpolateModeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Linear" => Ok(InterpolateMode::Linear),
            unknown => Err(UnknownInterpolateModeError(unknown.to_owned())),
        }
    }
}

/// Parameter binding to a node. This allows to animate a node based on the value of the parameter that owns it.
#[derive(Debug)]
pub struct Binding {
    pub node: InoxNodeUuid,
    pub is_set: Vec<Vec<bool>>,
    pub interpolate_mode: InterpolateMode,
    pub values: BindingValues,
}

#[derive(Debug)]
pub enum BindingValues {
    ZSort(Vec<Vec<f32>>),
    TransformTX(Vec<Vec<f32>>),
    TransformTY(Vec<Vec<f32>>),
    TransformSX(Vec<Vec<f32>>),
    TransformSY(Vec<Vec<f32>>),
    TransformRX(Vec<Vec<f32>>),
    TransformRY(Vec<Vec<f32>>),
    TransformRZ(Vec<Vec<f32>>),
    Deform(Vec<Vec<Vec<Vec2>>>),
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
