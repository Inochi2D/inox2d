use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;

use glow::HasContext;

use crate::mesh::SMesh;
use crate::model::ModelTexture;
use crate::nodes::drawable::{BlendMode, Mask};
use crate::nodes::node::{Node, NodeUuid};
use crate::nodes::node_tree::NodeTree;
use crate::nodes::part::Part;

use self::node_renderers::composite_renderer::CompositeRenderer;
use self::node_renderers::part_renderer::PartRenderer;
use self::texture::load_texture;
use self::vbo::Vbo;

pub mod node_renderers;
pub mod shader;
pub mod texture;
pub mod vbo;

pub trait NodeRenderer {
    type Node: Node;

    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node);
}

// I don't know if this is clever or yet another horrible workaround.
fn erase_node_renderer<R: NodeRenderer>(node_renderer: R) -> impl Fn(&OpenglRenderer, &dyn Node) {
    move |renderer, node| {
        if let Some(node) = node.as_any().downcast_ref() {
            node_renderer.render(renderer, node);
        }
    }
}

#[derive(Default, Clone)]
pub struct GlCache {
    pub prev_program: Option<glow::NativeProgram>,
    pub prev_stencil: bool,
    pub prev_blend_mode: Option<BlendMode>,
    pub prev_texture: Option<glow::NativeTexture>,
    pub prev_masks: Vec<Mask>,
}

impl GlCache {
    pub fn update_program(&mut self, program: glow::NativeProgram) -> bool {
        if let Some(prev_program) = self.prev_program.replace(program) {
            prev_program != program
        } else {
            true
        }
    }

    pub fn update_stencil(&mut self, stencil: bool) -> bool {
        if self.prev_stencil != stencil {
            self.prev_stencil = stencil;
            true
        } else {
            false
        }
    }

    pub fn update_blend_mode(&mut self, blend_mode: BlendMode) -> bool {
        if let Some(prev_blend_mode) = self.prev_blend_mode.replace(blend_mode) {
            prev_blend_mode != blend_mode
        } else {
            true
        }
    }

    pub fn update_texture(&mut self, texture: glow::NativeTexture) -> bool {
        if let Some(prev_texture) = self.prev_texture.replace(texture) {
            prev_texture != texture
        } else {
            true
        }
    }

    pub fn update_masks(&mut self, masks: Vec<Mask>) -> bool {
        if self.prev_masks != masks {
            self.prev_masks = masks;
            true
        } else {
            false
        }
    }
}

type ErasedNodeRenderer = Box<dyn Fn(&OpenglRenderer, &dyn Node)>;

pub struct OpenglRenderer {
    pub gl: glow::Context,
    pub gl_cache: RefCell<GlCache>,
    pub nodes: NodeTree,
    pub vao: glow::NativeVertexArray,
    pub verts: Vbo<f32>,
    pub uvs: Vbo<f32>,
    pub deform: Vbo<f32>,
    pub ibo: Vbo<u16>,
    pub textures: Vec<glow::NativeTexture>,
    pub node_renderers: HashMap<TypeId, ErasedNodeRenderer>,
}

impl OpenglRenderer {
    pub fn new(gl: glow::Context, mut nodes: NodeTree, textures: Vec<ModelTexture>) -> Self {
        let vao = unsafe { gl.create_vertex_array() }.unwrap();

        let mut verts = Vbo::from(vec![-1., -1., -1., 1., 1., -1., 1., -1., -1., 1., 1., 1.]);
        let mut uvs = Vbo::from(vec![0., 0., 0., 1., 1., 0., 1., 0., 0., 1., 1., 1.]);
        let mut deform = Vbo::from(vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.]);

        let mut ibo = Vbo::new();

        let mut current_ibo_offset = 6;
        for node in nodes.arena.iter_mut() {
            if let Some(node) = node.get_mut().as_any_mut().downcast_mut::<Part>() {
                let smesh = SMesh::from(&node.mesh);

                let num_verts = smesh.vertices.0.len();
                assert_eq!(num_verts, smesh.uvs.0.len());

                node.start_indice = ibo.len() as u16;
                // node.start_deform = current_ibo_offset * 2;

                verts.extend_from_slice(smesh.vertices.0.as_slice());
                uvs.extend_from_slice(smesh.uvs.0.as_slice());
                deform.extend_from_slice(vec![0.; num_verts].as_slice());
                ibo.extend(smesh.indices.iter().map(|index| index + current_ibo_offset));
                current_ibo_offset += (num_verts / 2) as u16;
            }
        }

        let textures: Vec<_> = textures
            .iter()
            .map(|texture| load_texture(&gl, &texture.data))
            .collect();

        let part_renderer = PartRenderer::new(&gl);
        let composite_renderer = CompositeRenderer::new(&gl);

        let mut renderer = OpenglRenderer {
            gl,
            gl_cache: RefCell::new(GlCache::default()),
            nodes,
            vao,
            verts,
            uvs,
            deform,
            ibo,
            textures,
            node_renderers: HashMap::new(),
        };

        renderer.register_node_renderer(part_renderer);
        renderer.register_node_renderer(composite_renderer);

        renderer.upload_buffers();
        renderer
    }

    pub fn register_node_renderer<N, R>(&mut self, renderer: R)
    where
        N: Node + 'static,
        R: NodeRenderer<Node = N> + 'static,
    {
        let tag = TypeId::of::<N>();
        let erased = erase_node_renderer(renderer);
        self.node_renderers.insert(tag, Box::new(erased));
    }

    fn upload_buffers(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.bind_vertex_array(Some(self.vao));

            self.verts.upload(gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(0);

            self.uvs.upload(gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(1);

            self.deform
                .upload(gl, glow::ARRAY_BUFFER, glow::DYNAMIC_DRAW);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(2);

            self.ibo
                .upload(gl, glow::ELEMENT_ARRAY_BUFFER, glow::STATIC_DRAW);
        }
    }

    pub fn render_nodes(&self, sorted_nodes: &[NodeUuid]) {
        for &node_uuid in sorted_nodes {
            let node = self.nodes.get_node(node_uuid).unwrap();
            let node = node.as_ref();
            if let Some(render) = self.node_renderers.get(&node.type_id()) {
                render(self, node);
            }
        }
    }

    /////////////////////////////////////////

    pub fn use_program(&self, program: glow::NativeProgram) {
        if !self.gl_cache.borrow_mut().update_program(program) {
            return;
        }

        unsafe { self.gl.use_program(Some(program)) };
    }

    pub fn bind_texture(&self, texture: glow::NativeTexture) {
        if !self.gl_cache.borrow_mut().update_texture(texture) {
            return;
        }

        unsafe { self.gl.bind_texture(glow::TEXTURE_2D, Some(texture)) };
    }

    pub fn set_stencil(&self, stencil: bool) {
        if !self.gl_cache.borrow_mut().update_stencil(stencil) {
            return;
        }

        let gl = &self.gl;
        unsafe {
            if stencil {
                gl.enable(glow::STENCIL_TEST);
            } else {
                gl.disable(glow::STENCIL_TEST);
            }
        }
    }

    pub fn set_blend_mode(&self, mode: BlendMode) {
        if !self.gl_cache.borrow_mut().update_blend_mode(mode) {
            return;
        }

        let gl = &self.gl;
        unsafe {
            match mode {
                BlendMode::Normal => {
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
                }
                BlendMode::Multiply => {
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::DST_COLOR, glow::ONE_MINUS_SRC_ALPHA);
                }
                BlendMode::ColorDodge => {
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::DST_COLOR, glow::ONE);
                }
                BlendMode::LinearDodge => {
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::ONE, glow::ONE);
                }
                BlendMode::Screen => {
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_COLOR);
                }
                BlendMode::ClipToLower => {
                    gl.blend_equation(glow::FUNC_ADD);
                    gl.blend_func(glow::DST_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                }
                BlendMode::SliceFromLower => {
                    gl.blend_equation(glow::FUNC_SUBTRACT);
                    gl.blend_func(glow::ONE_MINUS_DST_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                }
            }
        }
    }

    pub fn clear(&self) {
        unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) };
    }
}
