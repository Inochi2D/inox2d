use crate::nodes::node::Node;
use crate::nodes::part::Part;

use super::{OpenglRenderer, shader};

pub trait NodeRenderer {
    type Node: Node;
    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node);
}

const VERTEX: &str = include_str!("../../../shaders/basic/basic.vert");
const FRAGMENT: &str = include_str!("../../../shaders/basic/basic-mask.frag");

#[derive(Debug, Clone)]
pub(super) struct PartResources {
    pub part_program: glow::NativeProgram,
}

impl PartResources {
    pub fn register(renderer: &mut OpenglRenderer) {
        let part_program = shader::compile(&renderer.gl, VERTEX, FRAGMENT).unwrap();
        renderer.register_node_resource::<Part, _>(PartResources { part_program });
    }
}

struct PartRenderer;

impl NodeRenderer for PartRenderer {
    type Node = Part;

    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node) {
        use glow::HasContext;

        let resources = renderer
            .get_node_resource::<Self::Node, PartResources>()
            .unwrap();
        renderer.use_program(resources.part_program);

        if !node.draw_state.masks.is_empty() {
            self.recompute_masks(renderer, node);
        }

        renderer.bind_texture(renderer.textures[node.textures[0]]);
        renderer.set_blend_mode(node.draw_state.blend_mode);

        let trans = self.trans(renderer, node);
        let gl = &renderer.gl;
        unsafe {
            gl.uniform_3_f32_slice(renderer.locations.as_ref(), &trans.to_array());

            gl.draw_elements(
                glow::TRIANGLES,
                node.num_indices() as i32,
                glow::UNSIGNED_SHORT,
                (node.start_indice as i32) * 2,
            );
        }
    }
}

impl PartRenderer {
    fn recompute_masks(&self, renderer: &OpenglRenderer, node: &Part) {
        use glow::HasContext;
        use std::any::{Any, TypeId};

        let prev_masks = &renderer.gl_cache.borrow().prev_masks;
        if prev_masks == &node.draw_state.masks {
            return;
        }

        unsafe {
            renderer.set_stencil(true);
            {
                let gl = &renderer.gl;
                gl.color_mask(false, false, false, false);
                gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
                gl.stencil_func(glow::ALWAYS, 0xff, 0xff);
                gl.clear(glow::STENCIL_BUFFER_BIT);
            }

            for mask in node.draw_state.masks.iter() {
                let node_opt = renderer.nodes.get_node(mask.source);
                let node = node_opt.unwrap();
                if node.type_id() == TypeId::of::<Self>() {
                    node.render(renderer);
                }
            }

            let gl = &renderer.gl;
            gl.color_mask(true, true, true, true);
            gl.stencil_func(glow::EQUAL, 0xff, 0xff);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
        }

        let prev_masks = &mut renderer.gl_cache.borrow_mut().prev_masks;
        *prev_masks = node.draw_state.masks.clone();
    }

    fn trans(&self, renderer: &OpenglRenderer, node: &Part) -> glam::Vec3 {
        use crate::nodes::composite::Composite;
        use std::any::{Any, TypeId};

        let mut trans = node.node_state.transform.translation;

        let mut current_uuid = node.node_state.uuid;
        while let Some(parent) = renderer.nodes.get_parent(current_uuid) {
            let (parent, parent_trans) = if parent.type_id() == TypeId::of::<Self>()
                || parent.type_id() == TypeId::of::<Composite>()
            {
                let pstate = parent.get_node_state();
                (pstate.uuid, pstate.transform.translation)
            } else {
                break;
            };
            trans += parent_trans;
            current_uuid = parent;
        }
        trans
    }
}
