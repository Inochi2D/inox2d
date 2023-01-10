pub mod gl_buffer;
pub mod shader;
pub mod shaders;
pub mod texture;

use std::io;

use glam::{uvec2, UVec2, Vec2, Vec3};
use glow::HasContext;
use image::ImageFormat;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::math::camera::Camera;
use crate::model::ModelTexture;
use crate::nodes::node::InoxNodeUuid;
use crate::nodes::node_data::InoxData;
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

pub struct OpenglRenderer<T = ()> {
    gl: glow::Context,
    pub camera: Camera,
    pub viewport: UVec2,

    vao: glow::NativeVertexArray,

    verts: GlBuffer<Vec2>,
    uvs: GlBuffer<Vec2>,
    deforms: GlBuffer<Vec2>,
    indices: GlBuffer<u16>,

    part_shader: PartShader,

    textures: Vec<Texture>,

    pub nodes: InoxNodeTree<T>,
    pub sorted_uuids: Vec<InoxNodeUuid>,
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
        for node_id in &sorted_uuids {
            let node = nodes.get_node(*node_id).unwrap();

            if let InoxData::Part(ref part) = node.data {
                let mesh = &part.mesh;
                verts.extend_from_slice(&mesh.vertices);
                uvs.extend_from_slice(&mesh.uvs);
                indices.extend_from_slice(&mesh.indices);
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

            vao,

            verts,
            uvs,
            deforms,
            indices,

            part_shader,

            textures: Vec::new(),

            nodes,
            sorted_uuids,
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
                    let img = read_tga(&mut io::Cursor::new(&mtex.data)).ok()?;
                    Some((
                        img.data,
                        img.header.width() as u32,
                        img.header.height() as u32,
                    ))
                } else {
                    let img_buf = image::load_from_memory_with_format(&mtex.data, mtex.format)
                        .map_err(TextureError::LoadData)
                        .ok()?
                        .into_rgba8();

                    Some((img_buf.to_vec(), img_buf.width(), img_buf.height()))
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

    pub fn draw_stuff(&self) {
        let mvp = self.camera.matrix(self.viewport.as_vec2());

        let first_node = self.nodes.get_node(self.sorted_uuids[0]).unwrap();
        let InoxData::Part(ref part) = first_node.data else {
            eprintln!("nope");
            return;
        };

        let gl = &self.gl;
        let part_shader = &self.part_shader;
        unsafe {
            part_shader.bind(gl);

            // textures
            self.textures[part.tex_albedo].bind_on(gl, 0);
            self.textures[part.tex_bumpmap].bind_on(gl, 1);
            self.textures[part.tex_emissive].bind_on(gl, 2);

            // vert uniforms
            part_shader.set_mvp(gl, mvp);
            part_shader.set_offset(gl, first_node.transform.translation.truncate());

            // frag uniforms
            part_shader.set_opacity(gl, 1.0);
            part_shader.set_mult_color(gl, Vec3::splat(1.0));
            part_shader.set_screen_color(gl, Vec3::splat(0.0));

            let count = part.mesh.indices.len();
            gl.draw_elements(glow::TRIANGLES, count as i32, glow::UNSIGNED_SHORT, 0);
        }
    }
}
