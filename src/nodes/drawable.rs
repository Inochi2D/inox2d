use glam::Vec3;
use serde::{Serialize, Deserialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawable {
    blend_mode: BlendMode,
    tint: Vec3,
    #[serde(rename = "screenTint")]
    screen_tint: Vec3,
    mask_threshold: f32,
    opacity: f32,
}