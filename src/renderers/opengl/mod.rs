use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use glow::HasContext;

use crate::nodes::drawable::{BlendMode, Mask};
use crate::nodes::node::{downcast_node, Node, NodeUuid};
use crate::nodes::node_tree::NodeTree;

use self::node_renderers::composite_renderer::CompositeRenderer;
use self::node_renderers::part_renderer::PartRenderer;
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
        if let Some(node) = downcast_node(node) {
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
            prev_program == program
        } else {
            true
        }
    }

    pub fn update_stencil(&mut self, stencil: bool) -> bool {
        if self.prev_stencil == stencil {
            false
        } else {
            self.prev_stencil = stencil;
            true
        }
    }

    pub fn update_blend_mode(&mut self, blend_mode: BlendMode) -> bool {
        if let Some(prev_blend_mode) = self.prev_blend_mode.replace(blend_mode) {
            prev_blend_mode == blend_mode
        } else {
            true
        }
    }

    pub fn update_texture(&mut self, texture: glow::NativeTexture) -> bool {
        if let Some(prev_texture) = self.prev_texture.replace(texture) {
            prev_texture == texture
        } else {
            true
        }
    }

    pub fn update_masks(&mut self, masks: Vec<Mask>) -> bool {
        if self.prev_masks == masks {
            false
        } else {
            self.prev_masks = masks;
            true
        }
    }
}

type ErasedNodeRenderer = Box<dyn Fn(&OpenglRenderer, &dyn Node)>;

pub struct OpenglRenderer {
    pub gl: glow::Context,
    pub gl_cache: RefCell<GlCache>,
    pub nodes: NodeTree,
    pub verts: Vbo<f32>,
    pub uvs: Vbo<f32>,
    pub deform: Vbo<f32>,
    pub ibo: Vbo<u16>,
    pub current_ibo_offset: u16,
    pub textures: Vec<glow::NativeTexture>,
    pub node_renderers: HashMap<TypeId, ErasedNodeRenderer>,
}

impl OpenglRenderer {
    pub fn new(gl: glow::Context, nodes: NodeTree, textures: Vec<glow::NativeTexture>) -> Self {
        let part_renderer = PartRenderer::new(&gl);
        let composite_renderer = CompositeRenderer::new(&gl);

        let verts = Vbo::from(vec![-1., -1., -1., 1., 1., -1., 1., -1., -1., 1., 1., 1.]);
        let uvs = Vbo::from(vec![0., 0., 0., 1., 1., 0., 1., 0., 0., 1., 1., 1.]);
        let deform = Vbo::from(vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.]);

        let mut renderer = OpenglRenderer {
            gl,
            gl_cache: RefCell::new(GlCache::default()),
            nodes,
            verts,
            uvs,
            deform,
            ibo: Vbo::new(),
            current_ibo_offset: 6,
            textures,
            node_renderers: HashMap::new(),
        };

        renderer.register_node_renderer(part_renderer);
        renderer.register_node_renderer(composite_renderer);
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
            if let Some(render) = self.node_renderers.get(&node.type_id()) {
                render(self, node.as_ref());
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
