use glam::Vec3;

use super::node::NodeUuid;

/// Blending modes
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum MaskMode {
    Mask,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    pub source: NodeUuid,
    mode: MaskMode,
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
