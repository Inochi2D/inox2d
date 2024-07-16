use glow::HasContext;

use inox2d::render::RenderCtx;

use super::OpenglRendererError;

unsafe fn upload_array_to_gl<T>(gl: &glow::Context, array: &[T], target: u32, usage: u32) -> glow::Buffer {
	let bytes: &[u8] = core::slice::from_raw_parts(array.as_ptr() as *const u8, std::mem::size_of_val(array));
	let buffer = gl.create_buffer().unwrap();
	gl.bind_buffer(target, Some(buffer));
	gl.buffer_data_u8_slice(target, bytes, usage);
	buffer
}

unsafe fn reupload_array_to_gl<T>(gl: &glow::Context, array: &[T], target: u32, start_idx: usize, end_idx: usize) {
	let slice = &array[start_idx..end_idx];
	let bytes: &[u8] = core::slice::from_raw_parts(slice.as_ptr() as *const u8, core::mem::size_of_val(slice));
	gl.buffer_sub_data_u8_slice(target, start_idx as i32, bytes);
}

pub trait RenderCtxOpenglExt {
	unsafe fn setup_gl_buffers(
		&self,
		gl: &glow::Context,
		vao: glow::VertexArray,
	) -> Result<glow::Buffer, OpenglRendererError>;
	unsafe fn upload_deforms_to_gl(&self, gl: &glow::Context, buffer: glow::Buffer);
}

impl RenderCtxOpenglExt for RenderCtx {
	/// Uploads the vertex and index buffers to OpenGL.
	///
	/// # Errors
	///
	/// This function will return an error if it couldn't create a vertex array.
	///
	/// # Safety
	///
	/// Only call this function once when loading a new puppet.
	unsafe fn setup_gl_buffers(
		&self,
		gl: &glow::Context,
		vao: glow::VertexArray,
	) -> Result<glow::Buffer, OpenglRendererError> {
		gl.bind_vertex_array(Some(vao));

		upload_array_to_gl(gl, &self.vertex_buffers.verts, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
		gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
		gl.enable_vertex_attrib_array(0);

		upload_array_to_gl(gl, &self.vertex_buffers.uvs, glow::ARRAY_BUFFER, glow::STATIC_DRAW);
		gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);
		gl.enable_vertex_attrib_array(1);

		let deform_buffer =
			upload_array_to_gl(gl, &self.vertex_buffers.deforms, glow::ARRAY_BUFFER, glow::DYNAMIC_DRAW);
		gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 0, 0);
		gl.enable_vertex_attrib_array(2);

		upload_array_to_gl(
			gl,
			&self.vertex_buffers.indices,
			glow::ELEMENT_ARRAY_BUFFER,
			glow::STATIC_DRAW,
		);

		Ok(deform_buffer)
	}

	/// # Safety
	///
	/// unsafe as initiating GL calls. can be safely called for multiple times,
	/// but only needed once after deform update and before rendering.
	unsafe fn upload_deforms_to_gl(&self, gl: &glow::Context, buffer: glow::Buffer) {
		gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

		reupload_array_to_gl(
			gl,
			&self.vertex_buffers.deforms,
			glow::ARRAY_BUFFER,
			0,
			self.vertex_buffers.deforms.len(),
		);
	}
}
