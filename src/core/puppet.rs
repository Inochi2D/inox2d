#![allow(dead_code)]

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

/// Terms of usage of the puppet.
#[derive(Clone, Debug, Default)]
pub struct PuppetUsageRights {
    /// Who is allowed to use the puppet?
    allowed_users: PuppetAllowedUsers,
    /// Whether violence content is allowed.
    allow_violence: bool,
    /// Whether sexual content is allowed.
    allow_sexual: bool,
    /// Whether commercial use is allowed.
    allow_commercial: bool,
    /// Whether a model may be redistributed.
    allow_redistribution: PuppetAllowedRedistribution,
    /// Whether a model may be modified.
    allow_modification: PuppetAllowedModification,
    /// Whether the author(s) must be attributed for use.
    require_attribution: bool,
}

/// Puppet meta information.
#[derive(Clone, Debug)]
pub struct PuppetMeta {
    /// Name of the puppet.
    name: String,
    /// Version of the Inochi2D spec that was used when creating this model.
    version: String,
    /// Rigger(s) of the puppet.
    rigger: String,
    /// Artist(s) of the puppet.
    artist: String,
    /// Usage Rights of the puppet.
    rights: PuppetUsageRights,
    /// Copyright string.
    copyright: String,
    /// URL of the license.
    license_url: String,
    /// Contact information of the first author.
    contact: String,
    /// Link to the origin of this puppet.
    reference: String,
    /// Texture ID of this puppet's thumbnail.
    thumbnail_id: Option<u32>,
    /// Whether the puppet should preserve pixel borders.
    /// This feature is mainly useful for puppets that use pixel art.
    preserve_pixels: bool,
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

#[derive(Clone, Debug)]
pub struct PuppetPhysics {
    pixels_per_meter: f32,
    gravity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PuppetId(usize);

#[derive(Clone, Debug)]
pub struct Puppet {
    id: PuppetId,
    // TODO
}