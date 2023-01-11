pub mod gl_buffer;
pub mod shader;
pub mod shaders;
pub mod texture;

use std::cell::RefCell;
use std::ops::Deref;
use std::{io, mem};

use glam::{uvec2, UVec2, Vec2};
use glow::HasContext;
use image::ImageFormat;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tracing::{debug, error};

use crate::math::camera::Camera;
use crate::model::ModelTexture;
use crate::nodes::node::{InoxNode, InoxNodeUuid};
use crate::nodes::node_data::{BlendMode, InoxData, Part};
use crate::nodes::node_tree::InoxNodeTree;
use crate::texture::tga::read_tga;

use self::gl_buffer::GlBuffer;
use self::shader::ShaderCompileError;
use self::shaders::PartShader;
use self::texture::{Texture, TextureError};

#[derive(Debug, thiserror::Error)]
#[error("Could not initialize OpenGL renderer: {0}")]
pub enum OpenglRendererError {
    ShaderCompile(#[from] ShaderCompileError),
    Opengl(String),
}

#[derive(Default, Clone)]
pub struct GlCache {
    pub camera: Option<Camera>,
    pub blend_mode: Option<BlendMode>,
    pub program: Option<glow::NativeProgram>,
    pub albedo: Option<usize>,
}

impl GlCache {
    pub fn update_camera(&mut self, camera: &Camera) -> bool {
        if let Some(prev_camera) = &mut self.camera {
            let mut changed = false;

            if prev_camera.position != camera.position {
                prev_camera.position = camera.position;
                changed = true;
            }
            if prev_camera.rotation != camera.rotation {
                prev_camera.rotation = camera.rotation;
                changed = true;
            }
            if prev_camera.scale != camera.scale {
                prev_camera.scale = camera.scale;
                changed = true;
            }

            changed
        } else {
            self.camera = Some(camera.clone());
            true
        }
    }

    pub fn update_blend_mode(&mut self, blend_mode: BlendMode) -> bool {
        if let Some(prev_mode) = self.blend_mode.replace(blend_mode) {
            prev_mode != blend_mode
        } else {
            true
        }
    }

    pub fn update_program(&mut self, program: glow::NativeProgram) -> bool {
        if let Some(prev_program) = self.program.replace(program) {
            prev_program != program
        } else {
            true
        }
    }

    pub fn update_albedo(&mut self, albedo: usize) -> bool {
        if let Some(prev_texture) = self.albedo.replace(albedo) {
            prev_texture != albedo
        } else {
            true
        }
    }
}

enum NodeToRender {
    Part {
        uuid: InoxNodeUuid,
        index_offset: u16,
    },
}

pub struct OpenglRenderer<T = ()> {
    gl: glow::Context,
    pub camera: Camera,
    pub viewport: UVec2,
    cache: RefCell<GlCache>,

    vao: glow::NativeVertexArray,

    verts: GlBuffer<Vec2>,
    uvs: GlBuffer<Vec2>,
    deforms: GlBuffer<Vec2>,
    indices: GlBuffer<u16>,

    part_shader: PartShader,

    textures: Vec<Texture>,

    pub nodes: InoxNodeTree<T>,
    nodes_to_render: Vec<NodeToRender>,
}

impl<T> OpenglRenderer<T> {
    pub fn new(
        gl: glow::Context,
        viewport: UVec2,
        nodes: InoxNodeTree<T>,
    ) -> Result<Self, OpenglRendererError> {
        unsafe { gl.viewport(0, 0, viewport.x as i32, viewport.y as i32) };

        let mut verts = GlBuffer::new();
        let mut uvs = GlBuffer::new();
        let mut indices = GlBuffer::new();

        let sorted_uuids = nodes.zsorted();
        let mut nodes_to_render = Vec::new();
        let mut index_offset = 0;
        let mut vert_offset = 0;
        for &uuid in &sorted_uuids {
            let node = nodes.get_node(uuid).unwrap();

            if let InoxData::Part(ref part) = node.data {
                let mesh = &part.mesh;
                verts.extend_from_slice(&mesh.vertices);
                uvs.extend_from_slice(&mesh.uvs);
                indices.extend(mesh.indices.iter().map(|index| index + vert_offset));

                nodes_to_render.push(NodeToRender::Part { uuid, index_offset });
                index_offset += mesh.indices.len() as u16;
                vert_offset += mesh.vertices.len() as u16;
            }
        }

        // Initialize deforms to 0
        let deforms = GlBuffer::from(vec![Vec2::ZERO; verts.len()]);

        // Initialize buffers
        let vao;
        unsafe {
            vao = gl
                .create_vertex_array()
                .map_err(OpenglRendererError::Opengl)?;
            gl.bind_vertex_array(Some(vao));

            verts.upload(&gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(0);

            uvs.upload(&gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(1);

            deforms.upload(&gl, glow::ARRAY_BUFFER, glow::DYNAMIC_DRAW);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(2);

            indices.upload(&gl, glow::ELEMENT_ARRAY_BUFFER, glow::STATIC_DRAW);
        }

        // Shaders
        let part_shader = PartShader::new(&gl)?;

        Ok(Self {
            gl,
            camera: Camera::default(),
            viewport,
            cache: RefCell::new(GlCache::default()),

            vao,

            verts,
            uvs,
            deforms,
            indices,

            part_shader,

            textures: Vec::new(),

            nodes,
            nodes_to_render,
        })
    }

    pub fn upload_model_textures(
        &mut self,
        model_textures: &[ModelTexture],
    ) -> Result<(), TextureError> {
        // decode textures in parallel
        let images = model_textures
            .par_iter()
            .filter_map(|mtex| {
                if mtex.format == ImageFormat::Tga {
                    match read_tga(&mut io::Cursor::new(&mtex.data)) {
                        Ok(img) => Some((
                            img.data,
                            img.header.width() as u32,
                            img.header.height() as u32,
                        )),
                        Err(e) => {
                            error!("{}", e);
                            None
                        }
                    }
                } else {
                    let img_buf = image::load_from_memory_with_format(&mtex.data, mtex.format)
                        .map_err(TextureError::LoadData);

                    match img_buf {
                        Ok(img_buf) => {
                            let img_buf = img_buf.into_rgba8();
                            Some((img_buf.to_vec(), img_buf.width(), img_buf.height()))
                        }
                        Err(e) => {
                            error!("{}", e);
                            None
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        // upload textures
        for (pixels, width, height) in images {
            let tex = texture::Texture::from_raw_pixels(&self.gl, &pixels, width, height)?;
            self.textures.push(tex);
        }

        Ok(())
    }

    pub fn resize(&mut self, x: u32, y: u32) {
        self.viewport = uvec2(x, y);
        unsafe { self.gl.viewport(0, 0, x as i32, y as i32) };
    }

    pub fn clear(&self) {
        unsafe { self.gl.clear(glow::COLOR_BUFFER_BIT) };
    }

    #[inline]
    fn bind_shader<S: Deref<Target = glow::NativeProgram>>(&self, shader: &S) {
        let program = **shader;
        unsafe { self.gl.use_program(Some(program)) };
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

    /// Updates the camera in the GL cache and returns whether it changed.
    fn update_camera(&self) -> bool {
        if !self.cache.borrow_mut().update_camera(&self.camera) {
            return false;
        }

        let matrix = self.camera.matrix(self.viewport.as_vec2());

        self.bind_shader(&self.part_shader);
        unsafe { self.part_shader.set_mvp(&self.gl, matrix) };

        true
    }

    /// Set blending mode. See `BlendMode` for supported blend modes.
    pub fn set_blend_mode(&self, blend_mode: BlendMode) {
        if !self.cache.borrow_mut().update_blend_mode(blend_mode) {
            return;
        }

        let gl = &self.gl;
        unsafe {
            match blend_mode {
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

    pub fn draw_model(&self) {
        self.update_camera();
        unsafe { self.gl.enable(glow::BLEND) };

        for ntr in &self.nodes_to_render {
            match ntr {
                NodeToRender::Part {
                    uuid,
                    index_offset: start_index,
                } => {
                    let node = self.nodes.get_node(*uuid).unwrap();
                    if let InoxData::Part(ref part) = node.data {
                        self.draw_part(node, part, *start_index);
                    }
                }
            }
        }
    }

    #[inline]
    fn bind_part_textures(&self, part: &Part) {
        if !self.cache.borrow_mut().update_albedo(part.tex_albedo) {
            return;
        }

        let gl = &self.gl;
        self.textures[part.tex_albedo].bind_on(gl, 0);
        self.textures[part.tex_bumpmap].bind_on(gl, 1);
        self.textures[part.tex_emissive].bind_on(gl, 2);
    }

    fn draw_part(&self, node: &InoxNode<T>, part: &Part, start_index: u16) {
        self.push_debug_group(&node.name);

        // Position of current node by adding up its ancestors' positions
        let offset = self
            .nodes
            .ancestors(node.uuid)
            .filter_map(|ancestor| self.nodes.arena.get(ancestor))
            .map(|node| node.get().transform.translation.truncate())
            .sum();

        self.bind_part_textures(part);
        self.set_blend_mode(part.draw_state.blend_mode);

        let gl = &self.gl;
        let part_shader = &self.part_shader;
        unsafe {
            self.bind_shader(part_shader);

            // vert uniforms
            part_shader.set_offset(gl, offset);

            // frag uniforms
            part_shader.set_opacity(gl, part.draw_state.opacity);
            part_shader.set_mult_color(gl, part.draw_state.tint);
            part_shader.set_screen_color(gl, part.draw_state.screen_tint);

            gl.draw_elements(
                glow::TRIANGLES,
                part.mesh.indices.len() as i32,
                glow::UNSIGNED_SHORT,
                start_index as i32 * mem::size_of::<u16>() as i32,
            );
        }

        self.pop_debug_group();
    }
}
