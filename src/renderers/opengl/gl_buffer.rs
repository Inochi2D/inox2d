use std::ops::{Deref, DerefMut};

use glam::{vec2, Vec2};
use glow::HasContext;

use crate::mesh::Mesh;

use super::OpenglRendererError;

pub struct GlBuffer<T>(Vec<T>);

impl<T> Deref for GlBuffer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for GlBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> GlBuffer<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from(buffer: Vec<T>) -> Self {
        Self(buffer)
    }

    pub fn upload(&self, gl: &glow::Context, target: u32, usage: u32) -> glow::Buffer {
        let slice = self.as_slice();
        unsafe {
            let bytes: &[u8] = core::slice::from_raw_parts(
                slice.as_ptr() as *const u8,
                slice.len() * core::mem::size_of::<T>(),
            );
            let buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(target, Some(buffer));
            gl.buffer_data_u8_slice(target, bytes, usage);

            buffer
        }
    }
}

impl<T> Default for GlBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct InoxGlBuffersBuilder {
    verts: GlBuffer<Vec2>,
    uvs: GlBuffer<Vec2>,
    indices: GlBuffer<u16>,
    offset_index: u16,
    offset_vert: u16,
}

impl InoxGlBuffersBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_quad() -> Self {
        #[rustfmt::skip]
        let verts = GlBuffer::from(vec![
            vec2(-1.0, -1.0),
            vec2(-1.0,  1.0),
            vec2( 1.0, -1.0),
            vec2( 1.0,  1.0),
        ]);

        #[rustfmt::skip]
        let uvs = GlBuffer::from(vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 0.0),
            vec2(1.0, 1.0),
        ]);

        #[rustfmt::skip]
        let indices = GlBuffer::from(vec![
            0, 1, 2,
            2, 1, 3,
        ]);

        Self {
            verts,
            uvs,
            indices,
            offset_index: 6,
            offset_vert: 4,
        }
    }

    /// adds the mesh's vertices and UVs to the buffers and returns its index offset.
    pub fn push(&mut self, mesh: &Mesh) -> u16 {
        self.verts.extend_from_slice(&mesh.vertices);
        self.uvs.extend_from_slice(&mesh.uvs);
        self.indices
            .extend(mesh.indices.iter().map(|index| index + self.offset_vert));

        let offset_index = self.offset_index;

        self.offset_index += mesh.indices.len() as u16;
        self.offset_vert += mesh.vertices.len() as u16;

        offset_index
    }

    /// Uploads the vertex and index buffers to OpenGL.
    ///
    /// # Errors
    ///
    /// This function will return an error if it couldn't create a vertex array.
    ///
    /// # Safety
    ///
    /// Only call this function once (probably).
    pub unsafe fn upload(self, gl: &glow::Context) -> Result<InoxGlBuffers, OpenglRendererError> {
        let vao = gl
            .create_vertex_array()
            .map_err(OpenglRendererError::Opengl)?;
        gl.bind_vertex_array(Some(vao));

        self.verts.upload(gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        self.uvs.upload(gl, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        // Initialize deforms to 0
        let deforms = GlBuffer::from(vec![Vec2::ZERO; self.verts.len()]);
        deforms.upload(gl, glow::ARRAY_BUFFER, glow::DYNAMIC_DRAW);
        gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(2);

        self.indices
            .upload(gl, glow::ELEMENT_ARRAY_BUFFER, glow::STATIC_DRAW);

        Ok(InoxGlBuffers {
            vao,
            // verts: self.verts,
            // uvs: self.uvs,
            // deforms,
            // indices: self.indices,
        })
    }
}

pub struct InoxGlBuffers {
    vao: glow::VertexArray,
    // verts: GlBuffer<Vec2>,
    // uvs: GlBuffer<Vec2>,
    // deforms: GlBuffer<Vec2>,
    // indices: GlBuffer<u16>,
}

impl InoxGlBuffers {
    /// Binds this buffer bundle's vertex array.
    pub fn bind(&self, gl: &glow::Context) {
        unsafe { gl.bind_vertex_array(Some(self.vao)) };
    }
}

impl Deref for InoxGlBuffers {
    type Target = glow::VertexArray;

    fn deref(&self) -> &Self::Target {
        &self.vao
    }
}
