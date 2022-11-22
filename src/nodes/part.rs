use serde::{Deserialize, Serialize};

use crate::mesh::Mesh;

#[cfg(feature = "opengl")]
use crate::renderers::opengl::OpenglRenderer;

use super::drawable::Drawable;
use super::node::{Node, NodeState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(flatten)]
    pub node_state: NodeState,
    #[serde(flatten)]
    pub draw_state: Drawable,
    pub mesh: Mesh,
    pub textures: [usize; 3],
    #[cfg(feature = "opengl")]
    pub start_indice: u16,
}

#[typetag::serde]
impl Node for Part {
    fn get_node_state(&self) -> &NodeState {
        &self.node_state
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        &mut self.node_state
    }

    #[cfg(feature = "opengl")]
    fn render(&self, renderer: &mut OpenglRenderer) {
        use glow::HasContext;

        let resources = renderer
            .get_node_resource::<Self, __::PartResources>()
            .unwrap();
        renderer.use_program(resources.part_program);

        if !self.draw_state.masks.is_empty() {
            self.recompute_masks(renderer);
        }

        renderer.bind_texture(renderer.textures[self.textures[0]]);
        renderer.set_blend_mode(self.draw_state.blend_mode);

        let trans = self.trans(renderer);
        let gl = &renderer.gl;
        unsafe {
            gl.uniform_3_f32_slice(renderer.locations.as_ref(), &trans.to_array());

            gl.draw_elements(
                glow::TRIANGLES,
                self.num_indices() as i32,
                glow::UNSIGNED_SHORT,
                (self.start_indice as i32) * 2,
            );
        }
    }
}

impl Part {
    #[allow(unused)]
    fn num_indices(&self) -> u16 {
        self.mesh.indices.len() as u16
    }

    #[cfg(feature = "opengl")]
    fn recompute_masks(&self, renderer: &mut OpenglRenderer) {
        use std::any::{Any, TypeId};
        use glow::HasContext;

        if renderer.prev_masks == self.draw_state.masks {
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

            for mask in self.draw_state.masks.iter() {
                let node_opt = renderer.nodes.get_node_mut(mask.source);
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

        renderer.prev_masks = self.draw_state.masks.clone();
    }

    #[cfg(feature = "opengl")]
    fn trans(&self, renderer: &OpenglRenderer) -> glam::Vec3 {
        use crate::nodes::composite::Composite;
        use std::any::{Any, TypeId};

        let mut trans = self.node_state.transform.translation;

        let mut current_uuid = self.node_state.uuid;
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

#[cfg(feature = "opengl")]
pub mod __ {
    use crate::renderers::opengl::shader;

    use super::*;

    const VERTEX: &str = include_str!("../../shaders/basic/basic.vert");
    const FRAGMENT: &str = include_str!("../../shaders/basic/basic-mask.frag");

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
}
