use glow::HasContext;

use crate::nodes::drawable::Mask;
use crate::nodes::node::Node;
use crate::nodes::part::Part;
use crate::renderers::opengl::NodeRenderer;
use crate::renderers::opengl::{shader, OpenglRenderer};

const VERTEX: &str = "#version 100
precision mediump float;
uniform vec2 trans;
attribute vec2 pos;
attribute vec2 uvs;
attribute vec2 deform;
varying vec2 texcoord;

void main() {
    vec2 pos2 = pos + trans + deform;
    pos2.y = -pos2.y;
    texcoord = uvs;
    gl_Position = vec4(pos2 / 3072.0, 0.0, 1.0);
}
";

const FRAGMENT: &str = "#version 100
precision mediump float;
uniform sampler2D texture;
varying vec2 texcoord;

void main() {
    vec4 color = texture2D(texture, texcoord);
    if (color.a < 0.05) {
        discard;
    }
    gl_FragColor = color;
}
";

#[derive(Debug, Clone)]
pub(crate) struct PartRenderer {
    part_program: glow::NativeProgram,
    u_trans: Option<glow::NativeUniformLocation>,
}

impl NodeRenderer for PartRenderer {
    type Node = Part;

    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node) {
        let name = &node.get_node_state().name;
        let gl = &renderer.gl;
        unsafe { gl.push_debug_group(glow::DEBUG_SOURCE_APPLICATION, 0, name) };

        #[cfg(feature = "owo")]
        let name = {
            use owo_colors::OwoColorize;
            name.magenta()
        };

        eprintln!("  Rendering part {name}");
        renderer.set_stencil(false);
        self.render_part(renderer, node);

        unsafe { gl.pop_debug_group() };
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
            gl.uniform_2_f32(self.u_trans.as_ref(), trans.x, trans.y);

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
            if let Some(part) = mask_node.as_any().downcast_ref() {
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

        for ancestor in renderer.nodes.ancestors(node.node_state.uuid).skip(1) {
            if let Some(node) = renderer.nodes.arena.get(ancestor) {
                trans += node.get().get_node_state().transform.translation;
            }
        }

        trans
    }
}
