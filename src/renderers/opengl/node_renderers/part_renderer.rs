use glow::HasContext;

use crate::nodes::node::downcast_node;
use crate::nodes::part::Part;
use crate::renderers::opengl::NodeRenderer;
use crate::renderers::opengl::{shader, OpenglRenderer};

const VERTEX: &str = include_str!("../../../../shaders/basic/basic.vert");
const FRAGMENT: &str = include_str!("../../../../shaders/basic/basic-mask.frag");

#[derive(Debug, Clone)]
pub(crate) struct PartRenderer {
    pub part_program: glow::NativeProgram,
}

impl NodeRenderer for PartRenderer {
    type Node = Part;

    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node) {
        renderer.use_program(self.part_program);

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
    pub(crate) fn new(gl: &glow::Context) -> Self {
        let part_program = shader::compile(gl, VERTEX, FRAGMENT).unwrap();
        Self { part_program }
    }

    fn recompute_masks(&self, renderer: &OpenglRenderer, node: &Part) {
        let prev_masks = &renderer.gl_cache.borrow().prev_masks;
        if prev_masks == &node.draw_state.masks {
            return;
        }

        renderer.set_stencil(true);
        let gl = &renderer.gl;
        unsafe {
            gl.color_mask(false, false, false, false);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
            gl.stencil_func(glow::ALWAYS, 0xff, 0xff);
            gl.clear(glow::STENCIL_BUFFER_BIT);
        }

        for mask in node.draw_state.masks.iter() {
            let node_opt = renderer.nodes.get_node(mask.source);
            let node = node_opt.unwrap();
            if let Some(part) = downcast_node(node.as_ref()) {
                self.render(renderer, part);
            }
        }

        unsafe {
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
