use glam::{Vec2, Vec3};

use crate::mesh::Mesh;
use crate::params::ParamUuid;
use crate::physics::pendulum::rigid::RigidPendulum;
use crate::physics::pendulum::spring::SpringPendulum;
use crate::physics::runge_kutta::PhysicsState;
use crate::texture::TextureId;

use super::InoxNodeUuid;

/// Blending mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Drawable {
	/// Checks whether the drawable has masks of mode `MaskMode::Mask`.
	pub fn has_masks(&self) -> bool {
		self.masks.iter().any(|mask| mask.mode == MaskMode::Mask)
	}

	/// Checks whether the drawable has masks of mode `MaskMode::Dodge`.
	pub fn has_dodge_masks(&self) -> bool {
		self.masks.iter().any(|mask| mask.mode == MaskMode::Dodge)
	}
}

#[derive(Debug, Clone)]
pub struct Part {
	pub draw_state: Drawable,
	pub mesh: Mesh,
	pub tex_albedo: TextureId,
	pub tex_emissive: TextureId,
	pub tex_bumpmap: TextureId,
}

#[derive(Debug, Clone)]
pub struct Composite {
	pub draw_state: Drawable,
}

// TODO: PhysicsModel should just be a flat enum with no physics state.
// There's no reason to store a state if we're not simulating anything.
// This can be fixed in the component refactor.
// (I didn't want to create yet another separate PhysicsCtx just for this.)

/// Physics model to use for simple physics
#[derive(Debug, Clone)]
pub enum PhysicsModel {
	/// Rigid pendulum
	RigidPendulum(PhysicsState<RigidPendulum>),

	/// Springy pendulum
	SpringPendulum(PhysicsState<SpringPendulum>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParamMapMode {
	AngleLength,
	XY,
}

#[derive(Debug, Clone)]
pub struct PhysicsProps {
	/// Gravity scale (1.0 = puppet gravity)
	pub gravity: f32,
	/// Pendulum/spring rest length (pixels)
	pub length: f32,
	/// Resonant frequency (Hz)
	pub frequency: f32,
	/// Angular damping ratio
	pub angle_damping: f32,
	/// Length damping ratio
	pub length_damping: f32,

	pub output_scale: Vec2,
}

impl Default for PhysicsProps {
	fn default() -> Self {
		Self {
			gravity: 1.,
			length: 1.,
			frequency: 1.,
			angle_damping: 0.5,
			length_damping: 0.5,
			output_scale: Vec2::ONE,
		}
	}
}

#[derive(Debug, Clone)]
pub struct SimplePhysics {
	pub param: ParamUuid,

	pub model_type: PhysicsModel,
	pub map_mode: ParamMapMode,

	pub props: PhysicsProps,

	/// Whether physics system listens to local transform only.
	pub local_only: bool,

	// TODO: same as above, this state shouldn't be here.
	// It is only useful when simulating physics.
	pub bob: Vec2,
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
