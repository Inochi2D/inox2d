use glam::Vec3;

use super::super::InoxNodeUuid;

/// If has this as a component, the node should render something
pub struct Drawable {
	pub blending: Blending,
	/// If Some, the node should consider masking when rendering
	pub masks: Option<Masks>,
}

pub struct Blending {
	pub mode: BlendMode,
	pub tint: Vec3,
	pub screen_tint: Vec3,
	pub opacity: f32,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum BlendMode {
	/// Normal blending mode.
	#[default]
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

impl BlendMode {
	pub const VALUES: [BlendMode; 7] = [
		BlendMode::Normal,
		BlendMode::Multiply,
		BlendMode::ColorDodge,
		BlendMode::LinearDodge,
		BlendMode::Screen,
		BlendMode::ClipToLower,
		BlendMode::SliceFromLower,
	];
}

pub struct Masks {
	pub threshold: f32,
	pub masks: Vec<Mask>,
}

impl Masks {
	/// Checks whether has masks of mode `MaskMode::Mask`.
	pub fn has_masks(&self) -> bool {
		self.masks.iter().any(|mask| mask.mode == MaskMode::Mask)
	}

	/// Checks whether has masks of mode `MaskMode::Dodge`.
	pub fn has_dodge_masks(&self) -> bool {
		self.masks.iter().any(|mask| mask.mode == MaskMode::Dodge)
	}
}

pub struct Mask {
	pub source: InoxNodeUuid,
	pub mode: MaskMode,
}

#[derive(PartialEq)]
pub enum MaskMode {
	/// The part should be masked by the drawables specified.
	Mask,
	/// The path should be dodge-masked by the drawables specified.
	Dodge,
}
