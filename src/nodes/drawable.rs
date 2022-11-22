use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::node::NodeUuid;

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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
enum MaskMode {
    Mask,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Mask {
    pub source: NodeUuid,
    mode: MaskMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawable {
    pub blend_mode: BlendMode,
    pub tint: Vec3,
    #[serde(rename = "screenTint")]
    pub screen_tint: Vec3,
    pub mask_threshold: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub masks: Vec<Mask>,
    pub opacity: f32,
}
