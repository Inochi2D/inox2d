use crate::node::data::{BlendMode, InterpolateMode, MaskMode, ParamMapMode};
use crate::puppet::{PuppetAllowedModification, PuppetAllowedRedistribution, PuppetAllowedUsers};

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown param map mode {0:?}")]
pub struct UnknownParamMapModeError(String);

impl TryFrom<&str> for ParamMapMode {
	type Error = UnknownBlendModeError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"AngleLength" => Ok(ParamMapMode::AngleLength),
			"XY" => Ok(ParamMapMode::AngleLength),
			a => todo!("Param map mode {} unimplemented", a),
		}
	}
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

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown allowed users {0:?}")]
pub struct UnknownPuppetAllowedUsersError(String);

impl TryFrom<&str> for PuppetAllowedUsers {
	type Error = UnknownPuppetAllowedUsersError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"OnlyAuthor" => Ok(PuppetAllowedUsers::OnlyAuthor),
			"OnlyLicensee" => Ok(PuppetAllowedUsers::OnlyLicensee),
			"Everyone" => Ok(PuppetAllowedUsers::Everyone),
			unknown => Err(UnknownPuppetAllowedUsersError(unknown.to_owned())),
		}
	}
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown allowed redistribution {0:?}")]
pub struct UnknownPuppetAllowedRedistributionError(String);

impl TryFrom<&str> for PuppetAllowedRedistribution {
	type Error = UnknownPuppetAllowedRedistributionError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"Prohibited" => Ok(PuppetAllowedRedistribution::Prohibited),
			"ViralLicense" => Ok(PuppetAllowedRedistribution::ViralLicense),
			"CopyleftLicense" => Ok(PuppetAllowedRedistribution::CopyleftLicense),
			unknown => Err(UnknownPuppetAllowedRedistributionError(unknown.to_owned())),
		}
	}
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Unknown allowed users {0:?}")]
pub struct UnknownPuppetAllowedModificationError(String);

impl TryFrom<&str> for PuppetAllowedModification {
	type Error = UnknownPuppetAllowedModificationError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"Prohibited" => Ok(PuppetAllowedModification::Prohibited),
			"AllowPersonal" => Ok(PuppetAllowedModification::AllowPersonal),
			"AllowRedistribute" => Ok(PuppetAllowedModification::AllowRedistribute),
			unknown => Err(UnknownPuppetAllowedModificationError(unknown.to_owned())),
		}
	}
}
