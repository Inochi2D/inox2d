use std::collections::HashMap;
use std::fmt;

use crate::nodes::node_tree::InoxNodeTree;
use crate::params::Param;

// See this issue so we can maybe remove the TryFrom implementations
// in the future: https://github.com/Peternator7/strum/issues/13
mod display;

/// Who is allowed to use the puppet?
#[derive(Clone, Copy, Debug, Default)]
pub enum PuppetAllowedUsers {
    /// Only the author(s) are allowed to use the puppet.
    #[default]
    OnlyAuthor,
    /// Only licensee(s) are allowed to use the puppet.
    OnlyLicensee,
    /// Everyone may use the model.
    Everyone,
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

/// Can the puppet be redistributed?
#[derive(Clone, Copy, Debug, Default)]
pub enum PuppetAllowedRedistribution {
    /// Redistribution is prohibited
    #[default]
    Prohibited,
    /// Redistribution is allowed, but only under the same license
    /// as the original.
    ViralLicense,
    /// Redistribution is allowed, and the puppet may be
    /// redistributed under a different license than the original.
    ///
    /// This goes in conjunction with modification rights.
    CopyleftLicense,
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

/// Can the puppet be modified?
#[derive(Clone, Copy, Debug, Default)]
pub enum PuppetAllowedModification {
    /// Modification is prohibited
    #[default]
    Prohibited,
    /// Modification is only allowed for personal use
    AllowPersonal,
    /// Modification is allowed with redistribution, see
    /// `allowed_redistribution` for redistribution terms.
    AllowRedistribute,
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

/// Terms of usage of the puppet.
#[derive(Clone, Debug, Default)]
pub struct PuppetUsageRights {
    /// Who is allowed to use the puppet?
    pub allowed_users: PuppetAllowedUsers,
    /// Whether violence content is allowed.
    pub allow_violence: bool,
    /// Whether sexual content is allowed.
    pub allow_sexual: bool,
    /// Whether commercial use is allowed.
    pub allow_commercial: bool,
    /// Whether a model may be redistributed.
    pub allow_redistribution: PuppetAllowedRedistribution,
    /// Whether a model may be modified.
    pub allow_modification: PuppetAllowedModification,
    /// Whether the author(s) must be attributed for use.
    pub require_attribution: bool,
}

fn allowed_bool(value: bool) -> &'static str {
    if value {
        "allowed"
    } else {
        "prohibited"
    }
}

/// Puppet meta information.
#[derive(Clone, Debug)]
pub struct PuppetMeta {
    /// Name of the puppet.
    pub name: Option<String>,
    /// Version of the Inochi2D spec that was used when creating this model.
    pub version: String,
    /// Rigger(s) of the puppet.
    pub rigger: Option<String>,
    /// Artist(s) of the puppet.
    pub artist: Option<String>,
    /// Usage Rights of the puppet.
    pub rights: Option<PuppetUsageRights>,
    /// Copyright string.
    pub copyright: Option<String>,
    /// URL of the license.
    pub license_url: Option<String>,
    /// Contact information of the first author.
    pub contact: Option<String>,
    /// Link to the origin of this puppet.
    pub reference: Option<String>,
    /// Texture ID of this puppet's thumbnail.
    pub thumbnail_id: Option<u32>,
    /// Whether the puppet should preserve pixel borders.
    /// This feature is mainly useful for puppets that use pixel art.
    pub preserve_pixels: bool,
}

fn writeln_opt<T: fmt::Display>(
    f: &mut fmt::Formatter<'_>,
    field_name: &str,
    opt: &Option<T>,
) -> fmt::Result {
    let field_name = format!("{:<17}", format!("{field_name}:"));
    if let Some(ref value) = opt {
        #[cfg(feature = "owo")]
        let value = {
            use owo_colors::OwoColorize;
            value.green()
        };
        writeln!(f, "{field_name}{value}")?;
    }
    Ok(())
}

impl Default for PuppetMeta {
    fn default() -> Self {
        Self {
            name: Default::default(),
            version: crate::INOCHI2D_SPEC_VERSION.to_owned(),
            rigger: Default::default(),
            artist: Default::default(),
            rights: Default::default(),
            copyright: Default::default(),
            license_url: Default::default(),
            contact: Default::default(),
            reference: Default::default(),
            thumbnail_id: Default::default(),
            preserve_pixels: Default::default(),
        }
    }
}

/// Global physics parameters for the puppet.
#[derive(Clone, Debug)]
pub struct PuppetPhysics {
    pub pixels_per_meter: f32,
    pub gravity: f32,
}

/// Inochi2D puppet.
#[derive(Debug)]
pub struct Puppet<T = ()> {
    pub meta: PuppetMeta,
    pub physics: PuppetPhysics,
    pub nodes: InoxNodeTree<T>,
    pub parameters: HashMap<String, Param>,
}

impl<T> Puppet<T> {
    pub fn get_param(&self, name: &str) -> Option<&Param> {
        self.parameters.get(name)
    }
}
