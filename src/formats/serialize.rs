use std::collections::BTreeMap;

use glam::Vec2;
use indextree::Arena;
use json::JsonValue;

use crate::math::transform::Transform;
use crate::mesh::{f32s_as_vec2s, Mesh};
use crate::nodes::node::{ExtInoxNode, InoxNode, InoxNodeUuid};
use crate::nodes::node_data::{
    BlendMode, Composite, Drawable, InoxData, Mask, MaskMode, Part, UnknownBlendModeError,
    UnknownMaskModeError,
};
use crate::nodes::node_tree::ExtInoxNodeTree;
use crate::nodes::physics::SimplePhysics;
use crate::puppet::{
    Binding, BindingValues, ExtPuppet, InterpolateMode, Param, Puppet, PuppetAllowedModification,
    PuppetAllowedRedistribution, PuppetAllowedUsers, PuppetMeta, PuppetPhysics, PuppetUsageRights,
    UnknownInterpolateModeError, UnknownPuppetAllowedModificationError,
    UnknownPuppetAllowedRedistributionError, UnknownPuppetAllowedUsersError,
};

use super::json::{JsonError, JsonObject, SerialExtend};

pub type InoxParseResult<T> = Result<T, InoxParseError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum InoxParseError {
    #[error(transparent)]
    JsonError(#[from] JsonError),
    #[error("Unknown node type {0:?}")]
    UnknownNodeType(String),
    #[error("Unknown param name {0:?}")]
    UnknownParamName(String),
    #[error("No albedo texture")]
    NoAlbedoTexture,
    #[error(transparent)]
    UnknownBlendMode(#[from] UnknownBlendModeError),
    #[error(transparent)]
    UnknownMaskMode(#[from] UnknownMaskModeError),
    #[error(transparent)]
    UnknownInterpolateMode(#[from] UnknownInterpolateModeError),
    #[error(transparent)]
    UnknownPuppetAllowedUsers(#[from] UnknownPuppetAllowedUsersError),
    #[error(transparent)]
    UnknownPuppetAllowedRedistribution(#[from] UnknownPuppetAllowedRedistributionError),
    #[error(transparent)]
    UnknownPuppetAllowedModification(#[from] UnknownPuppetAllowedModificationError),
    #[error("Expected even number of floats in list, got {0}")]
    OddNumberOfFloatsInList(usize),
}

impl InoxParseError {
    pub fn nested(self, key: &str) -> Self {
        match self {
            InoxParseError::JsonError(err) => InoxParseError::JsonError(err.nested(key)),
            _ => self,
        }
    }
}

fn nested<T>(key: &str, res: InoxParseResult<T>) -> InoxParseResult<T> {
    res.map_err(|e| e.nested(key))
}

pub fn deserialize_node(obj: &JsonObject) -> InoxParseResult<InoxNode> {
    deserialize_node_ext(obj, &default_deserialize_custom)
}

fn default_deserialize_custom<T>(node_type: &str, _obj: &JsonObject) -> InoxParseResult<T> {
    Err(InoxParseError::UnknownNodeType(node_type.to_owned()))
}

pub fn deserialize_node_ext<T>(
    obj: &JsonObject,
    deserialize_node_custom: &impl Fn(&str, &JsonObject) -> InoxParseResult<T>,
) -> InoxParseResult<ExtInoxNode<T>> {
    let node_type = obj.get_str("type")?;
    Ok(ExtInoxNode {
        uuid: InoxNodeUuid(obj.get_u32("uuid")?),
        name: obj.get_str("name")?.to_owned(),
        enabled: obj.get_bool("enabled")?,
        zsort: obj.get_f32("zsort")?,
        transform: nested(
            "transform",
            deserialize_transform(&obj.get_object("transform")?),
        )?,
        lock_to_root: obj.get_bool("lockToRoot")?,
        data: nested(
            "data",
            deserialize_node_data(node_type, obj, deserialize_node_custom),
        )?,
    })
}

fn deserialize_node_data<T>(
    node_type: &str,
    obj: &JsonObject,
    deserialize_custom: &impl Fn(&str, &JsonObject) -> InoxParseResult<T>,
) -> InoxParseResult<InoxData<T>> {
    Ok(match node_type {
        "Node" => InoxData::Node,
        "Part" => InoxData::Part(deserialize_part(obj)?),
        "Composite" => InoxData::Composite(deserialize_composite(obj)?),
        "SimplePhysics" => InoxData::SimplePhysics(deserialize_simple_physics(obj)?),
        node_type => InoxData::Custom((deserialize_custom)(node_type, obj)?),
    })
}

fn deserialize_part(obj: &JsonObject) -> InoxParseResult<Part> {
    let (tex_albedo, tex_emissive, tex_bumpmap) = {
        let textures = obj.get_list("textures")?;

        let tex_albedo = match textures
            .get(0)
            .ok_or(InoxParseError::NoAlbedoTexture)?
            .as_number()
        {
            Some(val) => val.try_into().map_err(|_| {
                InoxParseError::JsonError(
                    JsonError::ParseIntError("0".to_owned()).nested("textures"),
                )
            })?,
            None => return Err(InoxParseError::NoAlbedoTexture),
        };

        let tex_emissive = match textures.get(1).and_then(JsonValue::as_number) {
            Some(val) => val.try_into().map_err(|_| {
                InoxParseError::JsonError(
                    JsonError::ParseIntError("1".to_owned()).nested("textures"),
                )
            })?,
            None => 0,
        };

        let tex_bumpmap = match textures.get(2).and_then(JsonValue::as_number) {
            Some(val) => val.try_into().map_err(|_| {
                InoxParseError::JsonError(
                    JsonError::ParseIntError("2".to_owned()).nested("textures"),
                )
            })?,
            None => 0,
        };

        (tex_albedo, tex_emissive, tex_bumpmap)
    };

    Ok(Part {
        draw_state: deserialize_drawable(obj)?,
        mesh: nested("mesh", deserialize_mesh(&obj.get_object("mesh")?))?,
        tex_albedo,
        tex_emissive,
        tex_bumpmap,
        #[cfg(feature = "opengl")]
        start_indice: 0,
    })
}

fn deserialize_composite(obj: &JsonObject) -> InoxParseResult<Composite> {
    let draw_state = deserialize_drawable(obj)?;
    Ok(Composite { draw_state })
}

fn deserialize_simple_physics(obj: &JsonObject) -> InoxParseResult<SimplePhysics> {
    Ok(SimplePhysics {
        param: obj.get_u32("param")?,
        model_type: obj.get_str("model_type")?.to_owned(),
        map_mode: obj.get_str("map_mode")?.to_owned(),
        gravity: obj.get_f32("gravity")?,
        length: obj.get_f32("length")?,
        frequency: obj.get_f32("frequency")?,
        angle_damping: obj.get_f32("angle_damping")?,
        length_damping: obj.get_f32("length_damping")?,
        output_scale: obj.get_vec2("output_scale")?,
    })
}

fn deserialize_drawable(obj: &JsonObject) -> InoxParseResult<Drawable> {
    Ok(Drawable {
        blend_mode: BlendMode::try_from(obj.get_str("blend_mode")?)?,
        tint: obj.get_vec3("tint")?,
        screen_tint: obj.get_vec3("screenTint")?,
        mask_threshold: obj.get_f32("mask_threshold")?,
        masks: {
            if let Ok(masks) = obj.get_list("masks") {
                masks
                    .iter()
                    .filter_map(|mask| deserialize_mask(&JsonObject(mask.as_object()?)).ok())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        },
        opacity: obj.get_f32("opacity")?,
    })
}

fn deserialize_mesh(obj: &JsonObject) -> InoxParseResult<Mesh> {
    Ok(Mesh {
        vertices: nested("verts", deserialize_vec2s(obj.get_list("verts")?))?,
        uvs: nested("uvs", deserialize_vec2s(obj.get_list("uvs")?))?,
        indices: obj
            .get_list("indices")?
            .iter()
            .map_while(JsonValue::as_u16)
            .collect::<Vec<_>>(),
        origin: obj.get_vec2("origin")?,
    })
}

fn deserialize_mask(obj: &JsonObject) -> InoxParseResult<Mask> {
    Ok(Mask {
        source: InoxNodeUuid(obj.get_u32("source")?),
        mode: MaskMode::try_from(obj.get_str("mode")?)?,
    })
}

fn deserialize_transform(obj: &JsonObject) -> InoxParseResult<Transform> {
    let translation = obj.get_vec3("trans")?;
    let rotation = obj.get_vec3("rot")?;
    let scale = obj.get_vec2("scale")?;
    let pixel_snap = obj.get_bool("pixel_snap").unwrap_or_default();

    Ok(Transform::new()
        .with_translation(translation)
        .with_rotation(rotation)
        .with_scale(scale)
        .with_pixel_snap(pixel_snap))
}

fn deserialize_f32s(val: &[json::JsonValue]) -> Vec<f32> {
    val.iter().filter_map(JsonValue::as_f32).collect::<Vec<_>>()
}

fn deserialize_vec2s(vals: &[json::JsonValue]) -> InoxParseResult<Vec<Vec2>> {
    if vals.len() % 2 != 0 {
        return Err(InoxParseError::OddNumberOfFloatsInList(vals.len()));
    }

    let floats = deserialize_f32s(vals);
    let mut vertices = Vec::new();
    vertices.extend_from_slice(f32s_as_vec2s(&floats));

    Ok(vertices)
}

// Puppet deserialization

pub fn deserialize_puppet(val: &json::JsonValue) -> InoxParseResult<Puppet> {
    deserialize_puppet_ext(val, &default_deserialize_custom)
}

pub fn deserialize_puppet_ext<T>(
    val: &json::JsonValue,
    deserialize_node_custom: &impl Fn(&str, &JsonObject) -> InoxParseResult<T>,
) -> InoxParseResult<ExtPuppet<T>> {
    let Some(obj) = val.as_object() else {
        return Err(InoxParseError::JsonError(JsonError::ValueIsNotObject("(puppet)".to_owned())));
    };
    let obj = JsonObject(obj);

    Ok(ExtPuppet {
        meta: nested("meta", deserialize_puppet_meta(&obj.get_object("meta")?))?,
        physics: nested(
            "physics",
            deserialize_puppet_physics(&obj.get_object("physics")?),
        )?,
        nodes: nested(
            "nodes",
            deserialize_nodes(&obj.get_object("nodes")?, deserialize_node_custom),
        )?,
        parameters: deserialize_params(obj.get_list("param")?),
    })
}

fn deserialize_params(vals: &[json::JsonValue]) -> Vec<Param> {
    vals.iter()
        .map_while(|param| deserialize_param(&JsonObject(param.as_object()?)).ok())
        .collect()
}

fn deserialize_param(obj: &JsonObject) -> InoxParseResult<Param> {
    Ok(Param {
        uuid: obj.get_u32("uuid")?,
        name: obj.get_str("name")?.to_owned(),
        is_vec2: obj.get_bool("is_vec2")?,
        min: obj.get_vec2("min")?,
        max: obj.get_vec2("max")?,
        defaults: obj.get_vec2("defaults")?,
        axis_points: nested(
            "axis_points",
            deserialize_axis_points(obj.get_list("axis_points")?),
        )?,
        bindings: deserialize_bindings(obj.get_list("bindings")?),
    })
}

fn deserialize_bindings(vals: &[json::JsonValue]) -> Vec<Binding> {
    vals.iter()
        .filter_map(|binding| deserialize_binding(&JsonObject(binding.as_object()?)).ok())
        .collect()
}

fn deserialize_binding(obj: &JsonObject) -> InoxParseResult<Binding> {
    Ok(Binding {
        node: InoxNodeUuid(obj.get_u32("node")?),
        is_set: obj
            .get_list("isSet")?
            .iter()
            .map(|bools| bools.members().map_while(JsonValue::as_bool).collect())
            .collect(),
        interpolate_mode: InterpolateMode::try_from(obj.get_str("interpolate_mode")?)?,
        values: deserialize_binding_values(obj.get_str("param_name")?, obj.get_list("values")?)?,
    })
}

fn deserialize_binding_values(
    param_name: &str,
    values: &[JsonValue],
) -> InoxParseResult<BindingValues> {
    Ok(match param_name {
        "zSort" => BindingValues::ZSort(deserialize_inner_binding_values(values)),
        "transform.t.x" => BindingValues::TransformTX(deserialize_inner_binding_values(values)),
        "transform.t.y" => BindingValues::TransformTY(deserialize_inner_binding_values(values)),
        "transform.s.x" => BindingValues::TransformSX(deserialize_inner_binding_values(values)),
        "transform.s.y" => BindingValues::TransformSY(deserialize_inner_binding_values(values)),
        "transform.r.x" => BindingValues::TransformRX(deserialize_inner_binding_values(values)),
        "transform.r.y" => BindingValues::TransformRY(deserialize_inner_binding_values(values)),
        "transform.r.z" => BindingValues::TransformRZ(deserialize_inner_binding_values(values)),
        "deform" => BindingValues::Deform(
            values
                .iter()
                .enumerate()
                .filter_map(|(j, vals)| {
                    Some(
                        as_nested_list(j, vals)
                            .ok()?
                            .iter()
                            .enumerate()
                            .filter_map(|(i, vals)| {
                                deserialize_vec2s(as_nested_list(i, vals).ok()?).ok()
                            })
                            .collect(),
                    )
                })
                .collect(),
        ),
        param_name => return Err(InoxParseError::UnknownParamName(param_name.to_owned())),
    })
}

fn deserialize_inner_binding_values(values: &[JsonValue]) -> Vec<Vec<f32>> {
    values
        .iter()
        .enumerate()
        .filter_map(|(i, vals)| Some(deserialize_f32s(as_nested_list(i, vals).ok()?)))
        .collect()
}

fn as_nested_list(index: usize, val: &json::JsonValue) -> InoxParseResult<&[json::JsonValue]> {
    match val {
        json::JsonValue::Array(arr) => Ok(arr),
        _ => Err(InoxParseError::JsonError(JsonError::ValueIsNotList(
            index.to_string(),
        ))),
    }
}

fn deserialize_axis_points(vals: &[json::JsonValue]) -> InoxParseResult<[Vec<f32>; 2]> {
    let x_points = deserialize_f32s(as_nested_list(0, &vals[0])?);
    let y_points = deserialize_f32s(as_nested_list(1, &vals[1])?);
    Ok([x_points, y_points])
}

fn deserialize_nodes<T>(
    obj: &JsonObject,
    deserialize_node_custom: &impl Fn(&str, &JsonObject) -> InoxParseResult<T>,
) -> InoxParseResult<ExtInoxNodeTree<T>> {
    let mut arena = Arena::new();
    let mut uuids = BTreeMap::new();

    let root_node = deserialize_node_ext(obj, deserialize_node_custom)?;
    let root_uuid = root_node.uuid;
    let root = arena.new_node(root_node);
    uuids.insert(root_uuid, root);

    let mut node_tree = ExtInoxNodeTree { root, arena, uuids };

    for (i, child) in obj.get_list("children").unwrap_or(&[]).iter().enumerate() {
        let Some(child) = child.as_object() else {
            return Err(InoxParseError::JsonError(JsonError::ValueIsNotObject(format!("children[{i}]"))))
        };

        let child_id =
            deserialize_nodes_rec(&JsonObject(child), deserialize_node_custom, &mut node_tree)
                .map_err(|e| e.nested(&format!("children[{i}]")))?;

        root.append(child_id, &mut node_tree.arena);
    }

    Ok(node_tree)
}

fn deserialize_nodes_rec<T>(
    obj: &JsonObject,
    deserialize_node_custom: &impl Fn(&str, &JsonObject) -> InoxParseResult<T>,
    node_tree: &mut ExtInoxNodeTree<T>,
) -> InoxParseResult<indextree::NodeId> {
    let node = deserialize_node_ext(obj, deserialize_node_custom)?;
    let uuid = node.uuid;
    let node_id = node_tree.arena.new_node(node);
    node_tree.uuids.insert(uuid, node_id);

    for (i, child) in obj.get_list("children").unwrap_or(&[]).iter().enumerate() {
        let Some(child) = child.as_object() else {
            return Err(InoxParseError::JsonError(JsonError::ValueIsNotObject(format!("children[{i}]"))))
        };
        let child_id =
            deserialize_nodes_rec(&JsonObject(child), deserialize_node_custom, node_tree)
                .map_err(|e| e.nested(&format!("children[{i}]")))?;

        node_id.append(child_id, &mut node_tree.arena);
    }

    Ok(node_id)
}

fn deserialize_puppet_physics(obj: &JsonObject) -> InoxParseResult<PuppetPhysics> {
    Ok(PuppetPhysics {
        pixels_per_meter: obj.get_f32("pixelsPerMeter")?,
        gravity: obj.get_f32("gravity")?,
    })
}

fn deserialize_puppet_meta(obj: &JsonObject) -> InoxParseResult<PuppetMeta> {
    Ok(PuppetMeta {
        name: obj.get_nullable_str("name")?.map(str::to_owned),
        version: obj.get_str("version")?.to_owned(),
        rigger: obj.get_nullable_str("rigger")?.map(str::to_owned),
        artist: obj.get_nullable_str("artist")?.map(str::to_owned),
        rights: match obj.get_object("rights").ok() {
            Some(ref rights) => Some(deserialize_puppet_usage_rights(rights)?),
            None => None,
        },
        copyright: obj.get_nullable_str("copyright")?.map(str::to_owned),
        license_url: obj.get_nullable_str("licenseURL")?.map(str::to_owned),
        contact: obj.get_nullable_str("contact")?.map(str::to_owned),
        reference: obj.get_nullable_str("reference")?.map(str::to_owned),
        thumbnail_id: obj.get_u32("thumbnailId").ok(),
        preserve_pixels: obj.get_bool("preservePixels")?,
    })
}

fn deserialize_puppet_usage_rights(obj: &JsonObject) -> InoxParseResult<PuppetUsageRights> {
    Ok(PuppetUsageRights {
        allowed_users: PuppetAllowedUsers::try_from(obj.get_str("allowed_users")?)?,
        allow_violence: obj.get_bool("allow_violence")?,
        allow_sexual: obj.get_bool("allow_sexual")?,
        allow_commercial: obj.get_bool("allow_commercial")?,
        allow_redistribution: PuppetAllowedRedistribution::try_from(
            obj.get_str("allow_redistribution")?,
        )?,
        allow_modification: PuppetAllowedModification::try_from(
            obj.get_str("allow_modification")?,
        )?,
        require_attribution: obj.get_bool("require_attribution")?,
    })
}
