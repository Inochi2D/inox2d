use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use glow::HasContext;

use crate::nodes::drawable::{BlendMode, Mask};
use crate::nodes::node::Node;
use crate::nodes::node_tree::NodeTree;

use self::node_renderers::part_renderer::PartRenderer;

pub mod node_renderers;
pub mod shader;
pub mod texture;
pub mod vbo;

pub trait NodeRenderer {
    type Node: Node;

    fn render(&self, renderer: &OpenglRenderer, node: &Self::Node);
}

#[derive(Default, Clone)]
pub struct GlCache {
    pub prev_program: Option<glow::NativeProgram>,
    pub prev_stencil: bool,
    pub prev_blend_mode: Option<BlendMode>,
    pub prev_texture: Option<glow::NativeTexture>,
    pub prev_masks: Vec<Mask>,
}

pub struct OpenglRenderer {
    pub gl: glow::Context,
    pub gl_cache: RefCell<GlCache>,
    pub nodes: NodeTree,
    pub textures: Vec<glow::NativeTexture>,
    pub locations: Option<glow::NativeUniformLocation>,
    pub node_renderers: HashMap<TypeId, Box<dyn Any>>,
}

impl OpenglRenderer {
    pub fn new(gl: glow::Context, nodes: NodeTree) -> Self {
        let mut renderer = OpenglRenderer {
            gl,
            gl_cache: RefCell::new(GlCache::default()),
            nodes,
            textures: Vec::new(),
            locations: None,
            node_renderers: HashMap::new(),
        };

        renderer.register_node_renderer(PartRenderer::new(&renderer.gl));
        renderer
    }

    pub fn register_node_renderer<N, R>(&mut self, renderer: R)
    where
        N: Node + 'static,
        R: NodeRenderer<Node = N> + 'static,
    {
        let tag = TypeId::of::<N>();
        self.node_renderers.insert(tag, Box::new(renderer));
    }

    pub fn get_node_renderer<N, R>(&self) -> Option<&R>
    where
        N: Node + 'static,
        R: NodeRenderer<Node = N> + 'static,
    {
        let tag = TypeId::of::<N>();
        if let Some(any) = self.node_renderers.get(&tag) {
            any.downcast_ref()
        } else {
            None
        }
    }

    pub fn use_program(&self, program: glow::NativeProgram) {
        let prev = &mut self.gl_cache.borrow_mut().prev_program;
        if *prev == Some(program) {
            return;
        }
        let gl = &self.gl;
        unsafe { gl.use_program(Some(program)) };
        *prev = Some(program);
    }

    pub fn bind_texture(&self, texture: glow::NativeTexture) {
        let prev = &mut self.gl_cache.borrow_mut().prev_texture;
        if *prev == Some(texture) {
            return;
        }
        let gl = &self.gl;
        unsafe { gl.bind_texture(glow::TEXTURE_2D, Some(texture)) };
        *prev = Some(texture);
    }

    pub fn set_stencil(&self, stencil: bool) {
        let prev = &mut self.gl_cache.borrow_mut().prev_stencil;
        if *prev == stencil {
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
        *prev = stencil;
    }

    pub fn set_blend_mode(&self, mode: BlendMode) {
        let prev = &mut self.gl_cache.borrow_mut().prev_blend_mode;
        if *prev == Some(mode) {
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
        *prev = Some(mode);
    }
}
