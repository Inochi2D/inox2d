#![allow(dead_code)]

use glam::Vec2;
use serde::{Serialize, Deserialize};

use crate::nodes::node::NodeUuid;
use crate::nodes::node_tree::NodeTree;

/// Who is allowed to use the puppet?
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
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
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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

/// Puppet meta information.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(rename = "licenseURL")]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuppetPhysics {
    pixels_per_meter: f32,
    gravity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InterpolateMode {
    Linear,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BindingBase {
    node: NodeUuid,
    #[serde(rename = "isSet")]
    is_set: Vec<Vec<bool>>,
    interpolate_mode: InterpolateMode,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "param_name")]
pub enum Binding {
    #[serde(rename = "zSort")]
    ZSort {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "transform.t.x")]
    TransformTX {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "transform.t.y")]
    TransformTY {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "transform.s.x")]
    TransformSX {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "transform.s.y")]
    TransformSY {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "transform.r.x")]
    TransformRX {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "transform.r.y")]
    TransformRY {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "transform.r.z")]
    TransformRZ {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<f32>>,
    },
    #[serde(rename = "deform")]
    Deform {
        #[serde(flatten)]
        base: BindingBase,
        values: Vec<Vec<Vec<Vec2>>>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Param {
    uuid: u32,
    name: String,
    is_vec2: bool,
    min: Vec2,
    max: Vec2,
    defaults: Vec2,
    axis_points: [Vec<f32>; 2],
    bindings: Vec<Binding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Puppet {
    meta: PuppetMeta,
    physics: PuppetPhysics,
    nodes: NodeTree,
    #[serde(rename = "param")]
    parameters: Vec<Param>,
}