use std::cell::RefCell;
use std::sync::mpsc;

use glam::{uvec2, UVec2};
use glow::HasContext;

use crate::math::camera::Camera;
use crate::nodes::node::{ExtInoxNode, InoxNodeUuid};
use crate::nodes::node_data::{BlendMode, Composite, InoxData, Mask, Part};
use crate::nodes::node_tree::{ExtInoxNodeTree, InoxNodeTree};
use crate::texture::Texture;

use self::gl_buffer::GlBuffer;
use self::texture::load_texture;

pub mod gl_buffer;
pub mod shader;
pub mod texture;

const PART_VERT: &str = include_str!("../../../shaders/simplified/part.vert");
const PART_FRAG: &str = include_str!("../../../shaders/simplified/part.frag");
const COMPOSITE_VERT: &str = include_str!("../../../shaders/simplified/composite.vert");
const COMPOSITE_FRAG: &str = include_str!("../../../shaders/simplified/composite.frag");

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

/// Default OpenGL renderer. Use this if your puppet doesn't have any nodes besides the Inochi2D builtin ones.
pub type OpenglRenderer = ExtOpenglRenderer<(), DefaultCustomRenderer>;

/// Custom OpenGL sub-renderer for custom nodes.
///
/// # Example
///
/// ```rs
/// struct Square {
///     side: f32,
/// }
///
/// struct SquareRenderer {
///     color: Vec4,
/// }
///
/// impl CustomRenderer for SquareRenderer {
///     type NodeData = Square;
///
///     fn render(
///         &self,
///         _renderer: &ExtOpenglRenderer<Square, Self>,
///         _node: &ExtInoxNode<Square>,
///         node_data: &Square,
///     ) where
///         Self: Sized,
///     {
///         println!("Rendering a square with side {} using color {}", node_data.side, self.color);
///     }
/// }
/// ```
pub trait CustomRenderer {
    type NodeData;

    fn render(
        &self,
        renderer: &ExtOpenglRenderer<Self::NodeData, Self>,
        node: &ExtInoxNode<Self::NodeData>,
        node_data: &Self::NodeData,
    ) where
        Self: Sized;
}

pub struct DefaultCustomRenderer;

impl CustomRenderer for DefaultCustomRenderer {
    type NodeData = ();

    fn render(
        &self,
        _renderer: &OpenglRenderer,
        _node: &ExtInoxNode<Self::NodeData>,
        _node_data: &Self::NodeData,
    ) {
    }
}

/// Creates a default OpenGL renderer.
/// Use this if your puppet doesn't have any nodes besides the Inochi2D builtin ones.
pub fn opengl_renderer(gl: glow::Context, viewport: UVec2, nodes: InoxNodeTree) -> OpenglRenderer {
    ExtOpenglRenderer::new(gl, viewport, nodes, DefaultCustomRenderer)
}

/// Creates an extensible OpenGL renderer.
/// Use this if your puppet has custom nodes besides the Inochi2D builtin ones.
pub fn opengl_renderer_ext<T, R>(
    gl: glow::Context,
    viewport: UVec2,
    nodes: ExtInoxNodeTree<T>,
    custom_renderer: R,
) -> ExtOpenglRenderer<T, R>
where
    R: CustomRenderer<NodeData = T>,
{
    ExtOpenglRenderer::new(gl, viewport, nodes, custom_renderer)
}

/// Extensible OpenGL renderer. It accepts a `CustomRenderer` to render your custom nodes.
///
/// Use this if your puppet has custom nodes besides the Inochi2D builtin ones.
pub struct ExtOpenglRenderer<T, R>
where
    R: CustomRenderer<NodeData = T>,
{
    /// OpenGL context.
    pub gl: glow::Context,
    /// Cache to avoid making unnecessary OpenGL calls.
    pub gl_cache: RefCell<GlCache>,
    /// Tree of nodes to render.
    pub nodes: ExtInoxNodeTree<T>, // TODO: maybe make a light copy of it instead of owning it?

    /// Viewport of the renderer.
    viewport: UVec2,
    /// Camera of the renderer.
    pub camera: Camera,

    /// Single vertex array for all the vertex buffers of the renderer.
    vao: glow::NativeVertexArray,

    /// Vertex buffer. Used to store vertex positions from meshes.
    pub verts: GlBuffer<f32>,
    /// UV buffer. Used to store UVs from Inochi2D meshes.
    pub uvs: GlBuffer<f32>,
    /// Deform buffer. Used to store mesh deformations, eventually...?
    pub deform: GlBuffer<f32>,
    /// Index buffer.
    pub ibo: GlBuffer<u16>,

    // OpenGL variables for GlBuffers above, stored for proper destruction on drop.
    nb_verts: glow::NativeBuffer,
    nb_uvs: glow::NativeBuffer,
    nb_deform: glow::NativeBuffer,
    nb_ibo: glow::NativeBuffer,

    /// All textures from the model, uploaded to the GPU.
    textures: Vec<glow::NativeTexture>,

    /// Shader program to render Part nodes.
    part_program: glow::NativeProgram,
    /// Location of the `u_mvp` uniform for the Part shader program.
    u_mvp: glow::NativeUniformLocation,
    /// Location of the `u_trans` uniform for the Part shader program.
    u_trans: glow::NativeUniformLocation,

    /// Shader program to render Composite nodes.
    composite_program: glow::NativeProgram,
    /// Texture created to draw composite stuff on it.
    composite_texture: glow::NativeTexture,
    /// Framebuffer where composite drawing happens.
    composite_fbo: glow::NativeFramebuffer,

    /// Custom renderer.
    pub custom_renderer: R,
}

impl<T, R> ExtOpenglRenderer<T, R>
where
    R: CustomRenderer<NodeData = T>,
{
    fn new(
        gl: glow::Context,
        viewport: UVec2,
        mut nodes: ExtInoxNodeTree<T>,
        render_custom: R,
    ) -> Self {
        // Set initial viewport size
        unsafe { gl.viewport(0, 0, viewport.x as i32, viewport.y as i32) };

        // Setup batch rendering of nodes
        let vao = unsafe { gl.create_vertex_array() }.unwrap();

        let mut verts = GlBuffer::from(vec![-1., -1., -1., 1., 1., -1., 1., -1., -1., 1., 1., 1.]);
        let mut uvs = GlBuffer::from(vec![0., 0., 0., 1., 1., 0., 1., 0., 0., 1., 1., 1.]);
        let mut deform = GlBuffer::from(vec![0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.]);

        let mut ibo = GlBuffer::new();

        let mut current_ibo_offset = 6;
        for node in nodes.arena.iter_mut() {
            if let InoxData::Part(ref mut part) = node.get_mut().data {
                let mesh = &part.mesh;

                let num_verts = mesh.vertices.len();
                assert_eq!(num_verts, mesh.uvs.len());

                part.start_indice = ibo.len() as u16;
                // node.start_deform = current_ibo_offset * 2;

                verts.extend_from_slice(mesh.vertices_as_f32s());
                uvs.extend_from_slice(mesh.uvs_as_f32s());
                deform.extend_from_slice(vec![0.; num_verts * 2].as_slice());
                ibo.extend(mesh.indices.iter().map(|index| index + current_ibo_offset));
                current_ibo_offset += num_verts as u16;
            }
        }

        // Part rendering
        let part_program = shader::compile(&gl, PART_VERT, PART_FRAG).unwrap();
        let u_mvp = unsafe { gl.get_uniform_location(part_program, "u_mvp") }.unwrap();
        let u_trans = unsafe { gl.get_uniform_location(part_program, "u_trans") }.unwrap();

        // Composite rendering
        let composite_program = shader::compile(&gl, COMPOSITE_VERT, COMPOSITE_FRAG).unwrap();

        let composite_texture;
        let composite_fbo;
        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.enable(glow::BLEND);
            gl.stencil_mask(0xff);

            composite_texture = texture::upload_texture(&gl, viewport.x, viewport.y, None);
            composite_fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(composite_fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(composite_texture),
                0,
            );
            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        };

        // upload buffers
        let nb_verts;
        let nb_uvs;
        let nb_deform;
        let nb_ibo;
        unsafe {
            gl.bind_vertex_array(Some(vao));

            nb_verts = verts.upload(&gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(0);

            nb_uvs = uvs.upload(&gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(1);

            nb_deform = deform.upload(&gl, glow::ARRAY_BUFFER, glow::DYNAMIC_DRAW);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 8, 0);
            gl.enable_vertex_attrib_array(2);

            nb_ibo = ibo.upload(&gl, glow::ELEMENT_ARRAY_BUFFER, glow::STATIC_DRAW);
        }

        ExtOpenglRenderer {
            gl,
            gl_cache: RefCell::new(GlCache::default()),
            nodes,
            viewport,
            camera: Camera::default(),
            vao,
            verts,
            uvs,
            deform,
            ibo,
            nb_verts,
            nb_uvs,
            nb_deform,
            nb_ibo,
            textures: Vec::new(),
            part_program,
            u_mvp,
            u_trans,
            composite_program,
            composite_texture,
            composite_fbo,
            custom_renderer: render_custom,
        }
    }

    pub fn viewport(&self) -> UVec2 {
        self.viewport
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.viewport = uvec2(width, height);

        let gl = &self.gl;
        unsafe { gl.viewport(0, 0, width as i32, height as i32) };

        // Resize composite texture
        self.bind_texture(self.composite_texture);
        unsafe {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );
        }
    }

    pub fn upload_textures(&mut self, rx: mpsc::Receiver<(usize, Texture)>, num_textures: usize) {
        let mut vec = vec![None; num_textures];
        while let Ok((i, tex)) = rx.recv() {
            let texture = load_texture(&self.gl, &tex);
            vec[i] = Some(texture);
        }
        self.textures = vec.into_iter().map(Option::unwrap).collect();
    }

    /// Pushes an OpenGL debug group.
    /// This is very useful to debug OpenGL calls per node with `apitrace`, as it will nest calls inside of labels,
    /// making it trivial to know which calls correspond to which nodes.
    ///
    /// It is a no-op on platforms that don't support it (only MacOS so far).
    #[inline]
    fn push_debug_group(&self, name: &str) {
        #[cfg(not(target_os = "macos"))]
        unsafe {
            self.gl
                .push_debug_group(glow::DEBUG_SOURCE_APPLICATION, 0, name);
        }
    }

    /// Pops the last OpenGL debug group.
    ///
    /// It is a no-op on platforms that don't support it (only MacOS so far).
    #[inline]
    fn pop_debug_group(&self) {
        #[cfg(not(target_os = "macos"))]
        unsafe {
            self.gl.pop_debug_group();
        }
    }

    pub fn render_nodes(&self, sorted_nodes: &[InoxNodeUuid]) {
        for &node_uuid in sorted_nodes {
            let node = self.nodes.get_node(node_uuid).unwrap();
            match node.data {
                InoxData::Part(ref part) => self.render_part(node, part, true),
                InoxData::Composite(ref composite) => self.render_composite(node, composite),
                InoxData::Custom(ref custom) => self.custom_renderer.render(self, node, custom),
                _ => (),
            }
        }
    }

    /// Renders a `Composite` node.
    ///
    /// It renders all its children in a separate framebuffer, and then draws the framebuffer with the composite's blend mode.
    fn render_composite(&self, node: &ExtInoxNode<T>, composite: &Composite) {
        let name = &node.name;
        let gl = &self.gl;

        self.push_debug_group(name);

        #[cfg(feature = "owo")]
        let name = {
            use owo_colors::OwoColorize;
            name.yellow()
        };

        eprintln!("Rendering composite {name}\n[");
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.composite_fbo));
            gl.clear(glow::COLOR_BUFFER_BIT);
            let children = self.nodes.get_children_uuids(node.uuid).unwrap_or_default();
            self.render_nodes(&children);

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.bind_texture(self.composite_texture);
            self.set_blend_mode(composite.draw_state.blend_mode);
            self.use_program(self.composite_program);
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
        eprintln!("]");

        self.pop_debug_group();
    }

    /// Renders a `Part` node.
    ///
    /// If the node has masks, it will render them before itself.
    fn render_part(&self, node: &ExtInoxNode<T>, part: &Part, disable_stencil: bool) {
        let name = &node.name;
        let gl = &self.gl;

        self.push_debug_group(name);

        #[cfg(feature = "owo")]
        let name = {
            use owo_colors::OwoColorize;
            name.magenta()
        };

        eprintln!("  Rendering part {name}");
        if disable_stencil {
            self.set_stencil(false);
        }
        self.use_program(self.part_program);

        if !part.draw_state.masks.is_empty() {
            self.recompute_masks(&part.draw_state.masks);
        }

        self.bind_texture(self.textures[part.tex_albedo]);
        self.set_blend_mode(part.draw_state.blend_mode);

        let trans = self.trans(node);

        unsafe {
            // TODO: is the camera matrix worth caching?
            gl.uniform_matrix_4_f32_slice(
                Some(&self.u_mvp),
                false,
                self.camera.matrix(self.viewport.as_vec2()).as_ref(),
            );
            gl.uniform_2_f32(Some(&self.u_trans), trans.x, trans.y);

            gl.draw_elements(
                glow::TRIANGLES,
                part.num_indices() as i32,
                glow::UNSIGNED_SHORT,
                (part.start_indice as i32) * 2,
            );
        }

        self.pop_debug_group();
    }

    /// Directly renders a `Part` node's masks.
    ///
    /// Currently only `Part` nodes can be masks.
    fn recompute_masks(&self, masks: &[Mask]) {
        if self.gl_cache.borrow().prev_masks == masks {
            return;
        }

        self.set_stencil(true);
        let gl = &self.gl;
        unsafe {
            gl.color_mask(false, false, false, false);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
            gl.stencil_func(glow::ALWAYS, 0xff, 0xff);
            gl.clear(glow::STENCIL_BUFFER_BIT);
        }

        for mask in masks.iter() {
            let mask_node = self.nodes.get_node(mask.source).unwrap();
            if let InoxData::Part(ref part) = mask_node.data {
                self.render_part(mask_node, part, false);
            }
        }

        unsafe {
            gl.color_mask(true, true, true, true);
            gl.stencil_func(glow::EQUAL, 0xff, 0xff);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
        }

        self.gl_cache.borrow_mut().update_masks(masks.to_vec());
    }

    /// Calculates the absolute position of a node by summing the transform position of all its ancestors.
    fn trans(&self, node: &ExtInoxNode<T>) -> glam::Vec3 {
        let mut trans = node.transform.translation;

        for ancestor in self.nodes.ancestors(node.uuid).skip(1) {
            if let Some(node) = self.nodes.arena.get(ancestor) {
                trans += node.get().transform.translation;
            }
        }

        trans
    }

    /////////////////////////////////////////

    /// Use an OpenGL shader program.
    pub fn use_program(&self, program: glow::NativeProgram) {
        if !self.gl_cache.borrow_mut().update_program(program) {
            return;
        }

        unsafe { self.gl.use_program(Some(program)) };
    }

    /// Bind an OpenGL texture.
    pub fn bind_texture(&self, texture: glow::NativeTexture) {
        if !self.gl_cache.borrow_mut().update_texture(texture) {
            return;
        }

        unsafe { self.gl.bind_texture(glow::TEXTURE_2D, Some(texture)) };
    }

    /// Enable or disable stencil.
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

    /// Set blending mode. See `BlendMode` for supported blend modes.
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

    /// Clears the framebuffer for the next frame.
    pub fn clear(&self) {
        unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) };
    }
}

impl<T, R> Drop for ExtOpenglRenderer<T, R>
where
    R: CustomRenderer<NodeData = T>,
{
    fn drop(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.delete_vertex_array(self.vao);

            gl.delete_buffer(self.nb_verts);
            gl.delete_buffer(self.nb_uvs);
            gl.delete_buffer(self.nb_deform);
            gl.delete_buffer(self.nb_ibo);

            for &texture in &self.textures {
                gl.delete_texture(texture);
            }

            gl.delete_program(self.part_program);

            gl.delete_program(self.composite_program);
            gl.delete_texture(self.composite_texture);
            gl.delete_framebuffer(self.composite_fbo);
        }
    }
}
