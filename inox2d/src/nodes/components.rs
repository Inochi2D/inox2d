use std::sync::Arc;

use glam::{Vec2, Vec3};

use crate::texture::TextureId;

use super::node::InoxNodeUuid;

#[derive(Clone, Debug)]
pub struct Name(pub Arc<str>);

#[derive(Clone, Debug)]
pub struct RootNode;

#[derive(Clone, Debug)]
pub struct InoxNode {
	pub zsort: f32,
	pub enabled: bool,
	pub lock_to_root: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Drawable {
	pub texture_offset: Vec2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
	Normal,
	Multiply,
	Screen,
	Overlay,
	Darken,
	Lighten,
	ColorDodge,
	LinearDodge,
	AddGlow,
	ColorBurn,
	HardLight,
	SoftLight,
	Subtract,
	Difference,
	Exclusion,
	Inverse,
	DestinationIn,
	ClipToLower,
	SliceFromLower,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown blending mode {0:?}")]
pub struct UnknownBlendModeError(String);

impl TryFrom<&str> for BlendMode {
	type Error = UnknownBlendModeError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"Normal" => Ok(Self::Normal),
			"Multiply" => Ok(Self::Multiply),
			"Screen" => Ok(Self::Screen),
			"Overlay" => Ok(Self::Overlay),
			"Darken" => Ok(Self::Darken),
			"Lighten" => Ok(Self::Lighten),
			"ColorDodge" => Ok(Self::ColorDodge),
			"LinearDodge" => Ok(Self::LinearDodge),
			"AddGlow" => Ok(Self::AddGlow),
			"ColorBurn" => Ok(Self::ColorBurn),
			"HardLight" => Ok(Self::HardLight),
			"SoftLight" => Ok(Self::SoftLight),
			"Subtract" => Ok(Self::Subtract),
			"Difference" => Ok(Self::Difference),
			"Exclusion" => Ok(Self::Exclusion),
			"Inverse" => Ok(Self::Inverse),
			"DestinationIn" => Ok(Self::DestinationIn),
			"ClipToLower" => Ok(Self::ClipToLower),
			"SliceFromLower" => Ok(Self::SliceFromLower),
			unknown => Err(UnknownBlendModeError(unknown.to_owned())),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaskMode {
	/// The part should be masked by the drawables specified.
	Mask,
	/// The path should be dodge-masked by the drawables specified.
	Dodge,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown mask mode {0:?}")]
pub struct UnknownMaskModeError(String);

impl TryFrom<&str> for MaskMode {
	type Error = UnknownMaskModeError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"Mask" => Ok(MaskMode::Mask),
			"DodgeMask" => Ok(MaskMode::Dodge),
			unknown => Err(UnknownMaskModeError(unknown.to_owned())),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mask {
	pub source: InoxNodeUuid,
	pub mode: MaskMode,
}

#[derive(Clone, Debug)]
pub struct Masks {
	pub threshold: f32,
	pub sources: Vec<Mask>,
}

impl Masks {
	/// Checks whether the drawable has masks of mode `MaskMode::Mask`.
	pub fn has_masks(&self) -> bool {
		self.sources.iter().any(|mask| mask.mode == MaskMode::Mask)
	}

	/// Checks whether the drawable has masks of mode `MaskMode::Dodge`.
	pub fn has_dodge_masks(&self) -> bool {
		self.sources.iter().any(|mask| mask.mode == MaskMode::Dodge)
	}
}

#[derive(Clone, Debug)]
pub struct Blending {
	pub tint_multiply: Vec3,
	pub tint_screen: Vec3,
	pub mode: BlendMode,
	pub opacity: f32,
}

#[derive(Clone, Debug)]
pub struct Part {
	pub tex_albedo: TextureId,
	pub tex_emissive: TextureId,
	pub tex_bumpmap: TextureId,

	pub emission_strength: f32,
	pub blending: Blending,
	pub masks: Masks,
}

#[derive(Clone, Debug)]
pub struct Composite {
	pub blending: Blending,
	pub masks: Masks,

	pub propagate_mesh_group: bool,
}
