use glam::Vec3;

use crate::mesh::Mesh;

use super::node::InoxNodeUuid;
use super::physics::SimplePhysics;

#[derive(Debug, Clone)]
pub struct Composite {
    pub(crate) draw_state: Drawable,
}

/// Blending mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown blend mode {0:?}")]
pub struct UnknownBlendModeError(String);

impl TryFrom<&str> for BlendMode {
    type Error = UnknownBlendModeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Normal" => Ok(BlendMode::Normal),
            "Multiply" => Ok(BlendMode::Multiply),
            "ColorDodge" => Ok(BlendMode::ColorDodge),
            "LinearDodge" => Ok(BlendMode::LinearDodge),
            "Screen" => Ok(BlendMode::Screen),
            "ClipToLower" => Ok(BlendMode::ClipToLower),
            "SliceFromLower" => Ok(BlendMode::SliceFromLower),
            unknown => Err(UnknownBlendModeError(unknown.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaskMode {
    Mask,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown mask mode {0:?}")]
pub struct UnknownMaskModeError(String);

impl TryFrom<&str> for MaskMode {
    type Error = UnknownMaskModeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Mask" => Ok(MaskMode::Mask),
            unknown => Err(UnknownMaskModeError(unknown.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    pub source: InoxNodeUuid,
    pub mode: MaskMode,
}

#[derive(Debug, Clone)]
pub struct Drawable {
    pub blend_mode: BlendMode,
    pub tint: Vec3,
    pub screen_tint: Vec3,
    pub mask_threshold: f32,
    pub masks: Vec<Mask>,
    pub opacity: f32,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub draw_state: Drawable,
    pub mesh: Mesh,
    pub tex_albedo: usize,
    pub tex_emissive: usize,
    pub tex_bumpmap: usize,
    #[cfg(feature = "opengl")]
    pub start_indice: u16,
    // start_deform: u16,
}

impl Part {
    pub fn num_indices(&self) -> u16 {
        self.mesh.indices.len() as u16
    }
}

#[derive(Debug, Clone)]
pub enum InoxData<T> {
    Node,
    Part(Part),
    Composite(Composite),
    SimplePhysics(SimplePhysics),
    Custom(T),
}

impl<T> InoxData<T> {
    pub fn is_node(&self) -> bool {
        matches!(self, InoxData::Node)
    }

    pub fn is_part(&self) -> bool {
        matches!(self, InoxData::Part(_))
    }

    pub fn is_composite(&self) -> bool {
        matches!(self, InoxData::Composite(_))
    }

    pub fn is_simple_physics(&self) -> bool {
        matches!(self, InoxData::SimplePhysics(_))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, InoxData::Custom(_))
    }

    pub fn data_type_name(&self) -> &'static str {
        match self {
            InoxData::Node => "Node",
            InoxData::Part(_) => "Part",
            InoxData::Composite(_) => "Composite",
            InoxData::SimplePhysics(_) => "SimplePhysics",
            InoxData::Custom(_) => "Custom",
        }
    }
}
