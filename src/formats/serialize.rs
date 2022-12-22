use std::collections::BTreeMap;

use glam::{Vec2, Vec3};
use indextree::Arena;
use json::JsonValue;

use crate::math::transform::Transform;
use crate::mesh::{f32s_as_vec2s, Mesh};
use crate::nodes::node::{ExtInoxNode, InoxNode, InoxNodeUuid};
use crate::nodes::node_data::{BlendMode, Composite, Drawable, InoxData, Mask, MaskMode, Part};
use crate::nodes::node_tree::ExtInoxNodeTree;
use crate::nodes::physics::SimplePhysics;
use crate::puppet::{
    Binding, BindingValues, ExtPuppet, InterpolateMode, Param, Puppet, PuppetAllowedModification,
    PuppetAllowedRedistribution, PuppetAllowedUsers, PuppetMeta, PuppetPhysics, PuppetUsageRights,
};

// TODO: use Result and return useful errors instead of Option
// This probably requires extending JsonValue with more functions...

trait SerialExtend {
    fn as_object(&self) -> Option<&json::object::Object>;
}

impl SerialExtend for JsonValue {
    fn as_object(&self) -> Option<&json::object::Object> {
        if let JsonValue::Object(transform) = self {
            Some(transform)
        } else {
            None
        }
    }
}

pub fn deserialize_node(obj: &json::object::Object) -> Option<InoxNode> {
    deserialize_node_ext(obj, &default_deserialize_custom)
}

fn default_deserialize_custom(_node_type: &str, _obj: &json::object::Object) -> Option<()> {
    Some(())
}

pub fn deserialize_node_ext<T>(
    obj: &json::object::Object,
    deserialize_node_custom: &impl Fn(&str, &json::object::Object) -> Option<T>,
) -> Option<ExtInoxNode<T>> {
    let node_type = obj.get("type")?.as_str()?;
    Some(ExtInoxNode {
        uuid: InoxNodeUuid(obj.get("uuid")?.as_u32()?),
        name: obj.get("name")?.as_str()?.to_owned(),
        enabled: obj.get("enabled")?.as_bool()?,
        zsort: obj.get("zsort")?.as_f32()?,
        transform: deserialize_transform(obj.get("transform")?.as_object()?)?,
        lock_to_root: obj.get("lockToRoot")?.as_bool()?,
        data: deserialize_node_data(node_type, obj, deserialize_node_custom)?,
    })
}

fn deserialize_node_data<T>(
    node_type: &str,
    obj: &json::object::Object,
    deserialize_custom: &impl Fn(&str, &json::object::Object) -> Option<T>,
) -> Option<InoxData<T>> {
    Some(match node_type {
        "Node" => InoxData::Node,
        "Part" => InoxData::Part(deserialize_part(obj)?),
        "Composite" => InoxData::Composite(deserialize_composite(obj)?),
        "SimplePhysics" => InoxData::SimplePhysics(deserialize_simple_physics(obj)?),
        node_type => InoxData::Custom((deserialize_custom)(node_type, obj)?),
    })
}

fn deserialize_part(obj: &json::object::Object) -> Option<Part> {
    let (tex_albedo, tex_emissive, tex_bumpmap) = {
        let mut textures = obj.get("textures")?.members();
        let tex_albedo = textures.next()?.as_usize()?;
        let tex_emissive = if let Some(tex) = textures.next() {
            tex.as_usize()?
        } else {
            0
        };
        let tex_bumpmap = if let Some(tex) = textures.next() {
            tex.as_usize()?
        } else {
            0
        };
        (tex_albedo, tex_emissive, tex_bumpmap)
    };

    Some(Part {
        draw_state: deserialize_drawable(obj)?,
        mesh: deserialize_mesh(obj.get("textures")?.as_object()?)?,
        tex_albedo,
        tex_emissive,
        tex_bumpmap,
        #[cfg(feature = "opengl")]
        start_indice: 0,
    })
}

fn deserialize_composite(obj: &json::object::Object) -> Option<Composite> {
    let draw_state = deserialize_drawable(obj)?;
    Some(Composite { draw_state })
}

fn deserialize_simple_physics(obj: &json::object::Object) -> Option<SimplePhysics> {
    Some(SimplePhysics {
        param: obj.get("param")?.as_u32()?,
        model_type: obj.get("model_type")?.as_str()?.to_owned(),
        map_mode: obj.get("map_mode")?.as_str()?.to_owned(),
        gravity: obj.get("gravity")?.as_f32()?,
        length: obj.get("length")?.as_f32()?,
        frequency: obj.get("frequency")?.as_f32()?,
        angle_damping: obj.get("angle_damping")?.as_f32()?,
        length_damping: obj.get("length_damping")?.as_f32()?,
        output_scale: deserialize_vec2(obj.get("output_scale")?)?,
    })
}

fn deserialize_drawable(obj: &json::object::Object) -> Option<Drawable> {
    Some(Drawable {
        blend_mode: BlendMode::try_from(obj.get("blend_mode")?.as_str()?).ok()?,
        tint: deserialize_vec3(obj.get("tint")?)?,
        screen_tint: deserialize_vec3(obj.get("screenTint")?)?,
        mask_threshold: obj.get("mask_threshold")?.as_f32()?,
        masks: {
            if let Some(masks) = obj.get("masks") {
                masks
                    .members()
                    .filter_map(|mask| deserialize_mask(mask.as_object()?))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        },
        opacity: obj.get("opacity")?.as_f32()?,
    })
}

fn deserialize_mesh(obj: &json::object::Object) -> Option<Mesh> {
    Some(Mesh {
        vertices: deserialize_vec2s(obj.get("verts")?)?,
        uvs: deserialize_vec2s(obj.get("uvs")?)?,
        indices: obj
            .get("indices")?
            .members()
            .map_while(JsonValue::as_u16)
            .collect::<Vec<_>>(),
        origin: deserialize_vec2(obj.get("origin")?)?,
    })
}

fn deserialize_mask(obj: &json::object::Object) -> Option<Mask> {
    Some(Mask {
        source: InoxNodeUuid(obj.get("source")?.as_u32()?),
        mode: MaskMode::try_from(obj.get("mode")?.as_str()?).ok()?,
    })
}

fn deserialize_transform(obj: &json::object::Object) -> Option<Transform> {
    let translation = deserialize_vec3(obj.get("trans")?)?;
    let rotation = deserialize_vec3(obj.get("rot")?)?;
    let scale = deserialize_vec2(obj.get("scale")?)?;

    let pixel_snap = if let Some(val) = obj.get("pixel_snap") {
        val.as_bool()?
    } else {
        false
    };

    Some(
        Transform::new()
            .with_translation(translation)
            .with_rotation(rotation)
            .with_scale(scale)
            .with_pixel_snap(pixel_snap),
    )
}

fn deserialize_f32s(val: &json::JsonValue) -> Vec<f32> {
    val.members()
        .map_while(JsonValue::as_f32)
        .collect::<Vec<_>>()
}

fn deserialize_vec2s(val: &json::JsonValue) -> Option<Vec<Vec2>> {
    let members = val.members();
    if members.len() % 2 != 0 {
        return None;
    }

    let floats = deserialize_f32s(val);
    let mut vertices = Vec::new();
    vertices.extend_from_slice(f32s_as_vec2s(&floats));

    Some(vertices)
}

fn deserialize_vec3(val: &json::JsonValue) -> Option<Vec3> {
    let mut members = val.members();
    if members.len() != 3 {
        return None;
    }

    let x = members.next()?.as_f32()?;
    let y = members.next()?.as_f32()?;
    let z = members.next()?.as_f32()?;
    Some(Vec3::new(x, y, z))
}

fn deserialize_vec2(val: &json::JsonValue) -> Option<Vec2> {
    let mut members = val.members();
    if members.len() != 2 {
        return None;
    }

    let x = members.next()?.as_f32()?;
    let y = members.next()?.as_f32()?;
    Some(Vec2::new(x, y))
}

// Puppet deserialization

pub fn deserialize_puppet(val: &json::JsonValue) -> Option<Puppet> {
    deserialize_puppet_ext(val, &default_deserialize_custom)
}

pub fn deserialize_puppet_ext<T>(
    val: &json::JsonValue,
    deserialize_node_custom: &impl Fn(&str, &json::object::Object) -> Option<T>,
) -> Option<ExtPuppet<T>> {
    let obj = val.as_object()?;
    Some(ExtPuppet {
        meta: deserialize_puppet_meta(obj.get("meta")?.as_object()?)?,
        physics: deserialize_puppet_physics(obj.get("physics")?.as_object()?)?,
        nodes: deserialize_nodes(obj.get("nodes")?.as_object()?, deserialize_node_custom)?,
        parameters: deserialize_params(obj.get("param")?),
    })
}

fn deserialize_params(val: &json::JsonValue) -> Vec<Param> {
    val.members()
        .map_while(|param| deserialize_param(param.as_object()?))
        .collect()
}

fn deserialize_param(obj: &json::object::Object) -> Option<Param> {
    Some(Param {
        uuid: obj.get("uuid")?.as_u32()?,
        name: obj.get("name")?.as_str()?.to_owned(),
        is_vec2: obj.get("is_vec2")?.as_bool()?,
        min: deserialize_vec2(obj.get("min")?)?,
        max: deserialize_vec2(obj.get("max")?)?,
        defaults: deserialize_vec2(obj.get("defaults")?)?,
        axis_points: deserialize_axis_points(obj.get("axis_points")?)?,
        bindings: deserialize_bindings(obj.get("bindings")?),
    })
}

fn deserialize_bindings(val: &JsonValue) -> Vec<Binding> {
    val.members()
        .map_while(|binding| deserialize_binding(binding.as_object()?))
        .collect()
}

fn deserialize_binding(obj: &json::object::Object) -> Option<Binding> {
    Some(Binding {
        node: InoxNodeUuid(obj.get("node")?.as_u32()?),
        is_set: obj
            .get("isSet")?
            .members()
            .map(|bools| bools.members().map_while(JsonValue::as_bool).collect())
            .collect(),
        interpolate_mode: InterpolateMode::try_from(obj.get("interpolate_mode")?.as_str()?).ok()?,
        values: deserialize_binding_values(obj.get("param_name")?.as_str()?, obj.get("values")?)?,
    })
}

fn deserialize_binding_values(param_name: &str, values: &JsonValue) -> Option<BindingValues> {
    Some(match param_name {
        "zSort" => BindingValues::ZSort(values.members().map(deserialize_f32s).collect()),
        "transform.t.x" => {
            BindingValues::TransformTX(values.members().map(deserialize_f32s).collect())
        }
        "transform.t.y" => {
            BindingValues::TransformTY(values.members().map(deserialize_f32s).collect())
        }
        "transform.s.x" => {
            BindingValues::TransformSX(values.members().map(deserialize_f32s).collect())
        }
        "transform.s.y" => {
            BindingValues::TransformSY(values.members().map(deserialize_f32s).collect())
        }
        "transform.r.x" => {
            BindingValues::TransformRX(values.members().map(deserialize_f32s).collect())
        }
        "transform.r.y" => {
            BindingValues::TransformRY(values.members().map(deserialize_f32s).collect())
        }
        "transform.r.z" => {
            BindingValues::TransformRZ(values.members().map(deserialize_f32s).collect())
        }
        "deform" => BindingValues::Deform(
            values
                .members()
                .map(|vecs| vecs.members().map_while(deserialize_vec2s).collect())
                .collect(),
        ),
        _ => return None,
    })
}

fn deserialize_axis_points(val: &json::JsonValue) -> Option<[Vec<f32>; 2]> {
    let mut members = val.members();
    let x_points = deserialize_f32s(members.next()?);
    let y_points = deserialize_f32s(members.next()?);
    Some([x_points, y_points])
}

fn deserialize_nodes<T>(
    obj: &json::object::Object,
    deserialize_node_custom: &impl Fn(&str, &json::object::Object) -> Option<T>,
) -> Option<ExtInoxNodeTree<T>> {
    let mut arena = Arena::new();
    let mut uuids = BTreeMap::new();

    let root_node = deserialize_node_ext(obj, deserialize_node_custom)?;
    let root_uuid = root_node.uuid;
    let root = arena.new_node(root_node);
    uuids.insert(root_uuid, root);

    let mut node_tree = ExtInoxNodeTree { root, arena, uuids };

    for child in obj.get("children")?.members() {
        deserialize_nodes_rec(child.as_object()?, deserialize_node_custom, &mut node_tree)?;
    }

    Some(node_tree)
}

fn deserialize_nodes_rec<T>(
    obj: &json::object::Object,
    deserialize_node_custom: &impl Fn(&str, &json::object::Object) -> Option<T>,
    node_tree: &mut ExtInoxNodeTree<T>,
) -> Option<InoxNodeUuid> {
    let node = deserialize_node_ext(obj, deserialize_node_custom)?;
    let uuid = node.uuid;
    let node_id = node_tree.arena.new_node(node);
    node_tree.uuids.insert(uuid, node_id);

    for child in obj.get("children")?.members() {
        deserialize_nodes_rec(child.as_object()?, deserialize_node_custom, node_tree)?;
    }

    Some(uuid)
}

fn deserialize_puppet_physics(obj: &json::object::Object) -> Option<PuppetPhysics> {
    Some(PuppetPhysics {
        pixels_per_meter: obj.get("pixelsPerMeter")?.as_f32()?,
        gravity: obj.get("gravity")?.as_f32()?,
    })
}

fn deserialize_puppet_meta(obj: &json::object::Object) -> Option<PuppetMeta> {
    Some(PuppetMeta {
        name: obj
            .get("name")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
        version: obj.get("name")?.as_str()?.to_owned(),
        rigger: obj
            .get("rigger")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
        artist: obj
            .get("artist")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
        rights: obj
            .get("rights")
            .and_then(|rights| deserialize_puppet_usage_rights(rights.as_object()?)),
        copyright: obj
            .get("copyright")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
        license_url: obj
            .get("licenseURL")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
        contact: obj
            .get("contact")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
        reference: obj
            .get("reference")
            .and_then(JsonValue::as_str)
            .map(str::to_owned),
        thumbnail_id: obj.get("thumbnailId").and_then(JsonValue::as_u32),
        preserve_pixels: obj.get("preservePixels")?.as_bool()?,
    })
}

fn deserialize_puppet_usage_rights(obj: &json::object::Object) -> Option<PuppetUsageRights> {
    Some(PuppetUsageRights {
        allowed_users: PuppetAllowedUsers::try_from(obj.get("allowed_users")?.as_str()?).ok()?,
        allow_violence: obj.get("allow_violence")?.as_bool()?,
        allow_sexual: obj.get("allow_sexual")?.as_bool()?,
        allow_commercial: obj.get("allow_commercial")?.as_bool()?,
        allow_redistribution: PuppetAllowedRedistribution::try_from(
            obj.get("allow_redistribution")?.as_str()?,
        )
        .ok()?,
        allow_modification: PuppetAllowedModification::try_from(
            obj.get("allow_modification")?.as_str()?,
        )
        .ok()?,
        require_attribution: obj.get("require_attribution")?.as_bool()?,
    })
}
