use glow::HasContext;

use crate::render::RenderCtx;

use super::OpenglRendererError;

impl RenderCtx {
    unsafe fn upload_array_to_gl<T>(gl: &glow::Context, array: &Vec<T>, target: u32, usage: u32) {
        let bytes: &[u8] = core::slice::from_raw_parts(
            array.as_ptr() as *const u8,
            array.len() * core::mem::size_of::<T>(),
        );
        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(target, Some(buffer));
        gl.buffer_data_u8_slice(target, bytes, usage);
    }

    unsafe fn reupload_array_to_gl<T>(
        gl: &glow::Context,
        array: &[T],
        target: u32,
        start_idx: usize,
        end_idx: usize,
    ) {
        let slice = &array[start_idx..end_idx];
        let bytes: &[u8] =
            core::slice::from_raw_parts(slice.as_ptr() as *const u8, core::mem::size_of_val(slice));
        gl.buffer_sub_data_u8_slice(target, start_idx as i32, bytes);
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
    pub unsafe fn setup_gl_buffers(
        &self,
        gl: &glow::Context,
    ) -> Result<glow::VertexArray, OpenglRendererError> {
        let vao = gl
            .create_vertex_array()
            .map_err(OpenglRendererError::Opengl)?;
        gl.bind_vertex_array(Some(vao));

        Self::upload_array_to_gl(
            gl,
            &self.vertex_buffers.verts,
            glow::ARRAY_BUFFER,
            glow::STATIC_DRAW,
        );
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        Self::upload_array_to_gl(
            gl,
            &self.vertex_buffers.uvs,
            glow::ARRAY_BUFFER,
            glow::STATIC_DRAW,
        );
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(1);

        Self::upload_array_to_gl(
            gl,
            &self.vertex_buffers.deforms,
            glow::ARRAY_BUFFER,
            glow::DYNAMIC_DRAW,
        );
        gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(2);

        Self::upload_array_to_gl(
            gl,
            &self.vertex_buffers.indices,
            glow::ELEMENT_ARRAY_BUFFER,
            glow::STATIC_DRAW,
        );

        Ok(vao)
    }

    /// # Safety
    ///
    /// unsafe as initiating GL calls. can be safely called for multiple times,
    /// but only needed once after deform update and before rendering.
    pub unsafe fn upload_deforms_to_gl(&self, gl: &glow::Context) {
        Self::reupload_array_to_gl(
            gl,
            &self.vertex_buffers.deforms,
            glow::ARRAY_BUFFER,
            0,
            self.vertex_buffers.deforms.len(),
        );
    }
}
