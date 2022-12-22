use glam::{Vec2, Vec3};
use json::JsonValue;

use crate::math::transform::Transform;
use crate::mesh::{f32s_as_vec2s, Mesh};
use crate::nodes::node::{ExtInoxNode, InoxNodeUuid, InoxNode};
use crate::nodes::node_data::{BlendMode, Composite, Drawable, InoxData, Mask, MaskMode, Part};
use crate::nodes::physics::SimplePhysics;

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
    deserialize_custom: &impl Fn(&str, &json::object::Object) -> Option<T>,
) -> Option<ExtInoxNode<T>> {
    let node_type = obj.get("type")?.as_str()?;
    Some(ExtInoxNode {
        uuid: InoxNodeUuid(obj.get("uuid")?.as_u32()?),
        name: obj.get("name")?.as_str()?.to_owned(),
        enabled: obj.get("enabled")?.as_bool()?,
        zsort: obj.get("zsort")?.as_f32()?,
        transform: deserialize_transform(obj.get("transform")?.as_object()?)?,
        lock_to_root: obj.get("lockToRoot")?.as_bool()?,
        data: deserialize_node_data(node_type, obj, deserialize_custom)?,
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

fn deserialize_vec2s(val: &json::JsonValue) -> Option<Vec<Vec2>> {
    let members = val.members();
    if members.len() % 2 != 0 {
        return None;
    }

    let floats = members.map_while(JsonValue::as_f32).collect::<Vec<_>>();
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
