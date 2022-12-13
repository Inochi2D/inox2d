use std::any::TypeId;

use glow::HasContext;

use crate::nodes::composite::Composite;
use crate::nodes::drawable::Mask;
use crate::nodes::node::downcast_node;
use crate::nodes::part::Part;
use crate::renderers::opengl::NodeRenderer;
use crate::renderers::opengl::{shader, OpenglRenderer};

const VERTEX: &str = include_str!("../../../../shaders/basic/basic.vert");
const FRAGMENT: &str = include_str!("../../../../shaders/basic/basic-mask.frag");

#[derive(Debug, Clone)]
pub(crate) struct PartRenderer {
    pub part_program: glow::NativeProgram,
    pub u_trans: Option<glow::NativeUniformLocation>,
}

impl NodeRenderer for PartRenderer {
    type Node = Part;

    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node) {
        renderer.set_stencil(false);
        self.render_part(renderer, node);
    }
}

impl PartRenderer {
    pub(crate) fn new(gl: &glow::Context) -> Self {
        let part_program = shader::compile(gl, VERTEX, FRAGMENT).unwrap();
        let u_trans = unsafe { gl.get_uniform_location(part_program, "trans") };

        Self {
            part_program,
            u_trans,
        }
    }

    fn render_part(&self, renderer: &OpenglRenderer, node: &Part) {
        renderer.use_program(self.part_program);

        if !node.draw_state.masks.is_empty() {
            self.recompute_masks(renderer, &node.draw_state.masks);
        }

        renderer.bind_texture(renderer.textures[node.textures[0]]);
        renderer.set_blend_mode(node.draw_state.blend_mode);

        let trans = self.trans(renderer, node);
        let gl = &renderer.gl;
        unsafe {
            gl.uniform_3_f32_slice(self.u_trans.as_ref(), &trans.to_array());

            gl.draw_elements(
                glow::TRIANGLES,
                node.num_indices() as i32,
                glow::UNSIGNED_SHORT,
                (node.start_indice as i32) * 2,
            );
        }
    }

    fn recompute_masks(&self, renderer: &OpenglRenderer, masks: &[Mask]) {
        if renderer.gl_cache.borrow().prev_masks == masks {
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

        for mask in masks.iter() {
            let mask_node = renderer.nodes.get_node(mask.source).unwrap();
            if let Some(part) = downcast_node(mask_node.as_ref()) {
                self.render_part(renderer, part);
            }
        }

        unsafe {
            gl.color_mask(true, true, true, true);
            gl.stencil_func(glow::EQUAL, 0xff, 0xff);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
        }

        renderer.gl_cache.borrow_mut().update_masks(masks.to_vec());
    }

    fn trans(&self, renderer: &OpenglRenderer, node: &Part) -> glam::Vec3 {
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
