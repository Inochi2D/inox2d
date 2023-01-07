#![allow(dead_code)]

use std::fmt;

use glam::Vec2;

use crate::nodes::node::InoxNodeUuid;
use crate::nodes::node_tree::InoxNodeTree;

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

impl fmt::Display for PuppetAllowedUsers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PuppetAllowedUsers::OnlyAuthor => "only author",
                PuppetAllowedUsers::OnlyLicensee => "only licensee",
                PuppetAllowedUsers::Everyone => "Everyone",
            }
        )
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

impl fmt::Display for PuppetAllowedRedistribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PuppetAllowedRedistribution::Prohibited => "prohibited",
                PuppetAllowedRedistribution::ViralLicense => "viral license",
                PuppetAllowedRedistribution::CopyleftLicense => "copyleft license",
            }
        )
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

impl fmt::Display for PuppetAllowedModification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PuppetAllowedModification::Prohibited => "prohibited",
                PuppetAllowedModification::AllowPersonal => "allow personal",
                PuppetAllowedModification::AllowRedistribute => "allow redistribute",
            }
        )
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

impl fmt::Display for PuppetUsageRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "| allowed users:  {}", self.allowed_users)?;
        writeln!(f, "| violence:       {}", allowed_bool(self.allow_violence))?;
        writeln!(f, "| sexual:         {}", allowed_bool(self.allow_sexual))?;
        writeln!(
            f,
            "| commercial:     {}",
            allowed_bool(self.allow_commercial)
        )?;
        writeln!(f, "| redistribution: {}", self.allow_redistribution)?;
        writeln!(f, "| modification:   {}", self.allow_modification)?;
        writeln!(
            f,
            "| attribution: {}",
            if self.require_attribution {
                "required"
            } else {
                "not required"
            }
        )
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

impl fmt::Display for PuppetMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name {
            Some(ref name) => writeln_opt(f, "Name", &Some(name))?,
            None => {
                let no_name = "(No Name)";
                #[cfg(feature = "owo")]
                let no_name = {
                    use owo_colors::OwoColorize;
                    no_name.dimmed()
                };
                writeln!(f, "{no_name}")?
            }
        }

        writeln_opt(f, "Version", &Some(&self.version))?;
        writeln_opt(f, "Rigger", &self.rigger)?;
        writeln_opt(f, "Artist", &self.artist)?;

        if let Some(ref rights) = self.rights {
            writeln!(f, "Rights:")?;
            #[cfg(feature = "owo")]
            let rights = {
                use owo_colors::OwoColorize;
                rights.yellow()
            };
            writeln!(f, "{rights}")?;
        }

        writeln_opt(f, "Copyright", &self.copyright)?;
        writeln_opt(f, "License URL", &self.license_url)?;
        writeln_opt(f, "Contact", &self.contact)?;
        writeln_opt(f, "Reference", &self.reference)?;
        writeln_opt(f, "Thumbnail ID", &self.thumbnail_id)?;

        writeln_opt(
            f,
            "Preserve pixels",
            &Some(if self.preserve_pixels { "yes" } else { "no" }),
        )
    }
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

/// Inochi2D puppet.
#[derive(Debug)]
pub struct Puppet<T = ()> {
    pub meta: PuppetMeta,
    pub physics: PuppetPhysics,
    pub nodes: InoxNodeTree<T>,
    pub parameters: Vec<Param>,
}
