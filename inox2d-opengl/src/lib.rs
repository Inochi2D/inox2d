mod gl_buffer;
mod shader;
mod shaders;
pub mod texture;

use std::cell::RefCell;
use std::mem;
use std::ops::Deref;

use glam::{uvec2, UVec2, Vec3};
use glow::HasContext;

use inox2d::math::camera::Camera;
use inox2d::model::Model;
use inox2d::node::{
	components::{BlendMode, Mask, MaskMode, Masks, TexturedMesh},
	drawables::{CompositeComponents, TexturedMeshComponents},
	InoxNodeUuid,
};
use inox2d::puppet::Puppet;
use inox2d::render::{CompositeRenderCtx, InoxRenderer, TexturedMeshRenderCtx};
use inox2d::texture::{decode_model_textures, TextureId};

use self::shader::ShaderCompileError;
use self::shaders::{CompositeMaskShader, CompositeShader, PartMaskShader, PartShader};
use self::texture::Texture;

use gl_buffer::{setup_gl_buffers, upload_deforms_to_gl};

#[derive(Debug, thiserror::Error)]
#[error("Could not initialize OpenGL renderer: {0}")]
pub enum OpenglRendererError {
	ShaderCompile(#[from] ShaderCompileError),
	Opengl(String),
}

#[derive(Default)]
struct GlCache {
	pub camera: Option<Camera>,
	pub viewport: Option<UVec2>,
	pub blend_mode: Option<BlendMode>,
	pub program: Option<glow::Program>,
	pub vao: Option<glow::VertexArray>,
	pub albedo: Option<TextureId>,
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

	pub fn update_viewport(&mut self, viewport: UVec2) -> bool {
		if let Some(prev_viewport) = self.viewport.replace(viewport) {
			prev_viewport != viewport
		} else {
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

	pub fn update_program(&mut self, program: glow::Program) -> bool {
		if let Some(prev_program) = self.program.replace(program) {
			prev_program != program
		} else {
			true
		}
	}

	pub fn update_vao(&mut self, vao: glow::VertexArray) -> bool {
		if let Some(prev_vao) = self.vao.replace(vao) {
			prev_vao != vao
		} else {
			true
		}
	}

	pub fn update_albedo(&mut self, albedo: TextureId) -> bool {
		if let Some(prev_texture) = self.albedo.replace(albedo) {
			prev_texture != albedo
		} else {
			true
		}
	}
}

pub struct OpenglRenderer {
	gl: glow::Context,
	support_debug_extension: bool,
	pub camera: Camera,
	pub viewport: UVec2,
	cache: RefCell<GlCache>,

	vao: glow::VertexArray,

	composite_framebuffer: glow::Framebuffer,
	cf_albedo: glow::Texture,
	cf_emissive: glow::Texture,
	cf_bump: glow::Texture,
	cf_stencil: glow::Texture,

	part_shader: PartShader,
	part_mask_shader: PartMaskShader,
	composite_shader: CompositeShader,
	composite_mask_shader: CompositeMaskShader,

	textures: Vec<Texture>,
}

impl OpenglRenderer {
	/// Pushes an OpenGL debug group.
	/// This is very useful to debug OpenGL calls per node with `apitrace`, as it will nest calls inside of labels,
	/// making it trivial to know which calls correspond to which nodes.
	///
	/// It is a no-op on platforms that don't support it (like Apple *OS).
	#[inline]
	fn push_debug_group(&self, name: &str) {
		if self.support_debug_extension {
			unsafe {
				self.gl.push_debug_group(glow::DEBUG_SOURCE_APPLICATION, 0, name);
			}
		}
	}

	/// Pops the last OpenGL debug group.
	///
	/// It is a no-op on platforms that don't support it (like Apple *OS).
	#[inline]
	fn pop_debug_group(&self) {
		if self.support_debug_extension {
			unsafe {
				self.gl.pop_debug_group();
			}
		}
	}

	/// Updates the camera in the GL cache and returns whether it changed.
	fn update_camera(&self) -> bool {
		{
			let mut cache = self.cache.borrow_mut();
			if !cache.update_camera(&self.camera) && !cache.update_viewport(self.viewport) {
				return false;
			}
		}

		let matrix = self.camera.matrix(self.viewport.as_vec2());

		self.bind_shader(&self.composite_shader);
		self.composite_shader.set_mvp(&self.gl, matrix);

		self.bind_shader(&self.composite_mask_shader);
		self.composite_mask_shader.set_mvp(&self.gl, matrix);

		true
	}

	/// Set blending mode. See `BlendMode` for supported blend modes.
	fn set_blend_mode(&self, blend_mode: BlendMode) {
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

	fn bind_shader<S: Deref<Target = glow::Program>>(&self, shader: &S) {
		let program = **shader;
		if !self.cache.borrow_mut().update_program(program) {
			return;
		}

		unsafe { self.gl.use_program(Some(program)) };
	}

	fn bind_part_textures(&self, part: &TexturedMesh) {
		if !self.cache.borrow_mut().update_albedo(part.tex_albedo) {
			return;
		}

		let gl = &self.gl;
		self.textures[part.tex_albedo.raw()].bind_on(gl, 0);
		self.textures[part.tex_bumpmap.raw()].bind_on(gl, 1);
		self.textures[part.tex_emissive.raw()].bind_on(gl, 2);
	}

	/// Clear the texture cache
	/// This one method missing made me pull my hair out for an entire month.
	fn clear_texture_cache(&self) {
		self.cache.borrow_mut().albedo = None;
	}

	unsafe fn attach_framebuffer_textures(&self) {
		let gl = &self.gl;
		gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.composite_framebuffer));

		gl.framebuffer_texture_2d(
			glow::FRAMEBUFFER,
			glow::COLOR_ATTACHMENT0,
			glow::TEXTURE_2D,
			Some(self.cf_albedo),
			0,
		);
		gl.framebuffer_texture_2d(
			glow::FRAMEBUFFER,
			glow::COLOR_ATTACHMENT1,
			glow::TEXTURE_2D,
			Some(self.cf_emissive),
			0,
		);
		gl.framebuffer_texture_2d(
			glow::FRAMEBUFFER,
			glow::COLOR_ATTACHMENT2,
			glow::TEXTURE_2D,
			Some(self.cf_bump),
			0,
		);
		gl.framebuffer_texture_2d(
			glow::FRAMEBUFFER,
			glow::DEPTH_STENCIL_ATTACHMENT,
			glow::TEXTURE_2D,
			Some(self.cf_stencil),
			0,
		);

		gl.bind_framebuffer(glow::FRAMEBUFFER, None);
	}

	pub fn resize(&mut self, w: u32, h: u32) {
		self.viewport = uvec2(w, h);

		let gl = &self.gl;
		unsafe {
			gl.viewport(0, 0, w as i32, h as i32);

			// Reupload composite framebuffer textures
			texture::upload_empty(gl, self.cf_albedo, w, h, glow::UNSIGNED_BYTE);
			texture::upload_empty(gl, self.cf_emissive, w, h, glow::FLOAT);
			texture::upload_empty(gl, self.cf_bump, w, h, glow::UNSIGNED_BYTE);

			gl.bind_texture(glow::TEXTURE_2D, Some(self.cf_stencil));
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::DEPTH24_STENCIL8 as i32,
				w as i32,
				h as i32,
				0,
				glow::DEPTH_STENCIL,
				glow::UNSIGNED_INT_24_8,
				None,
			);

			self.attach_framebuffer_textures();
		}
		self.update_camera();
	}

	pub fn clear(&self) {
		unsafe {
			self.gl.clear(glow::COLOR_BUFFER_BIT);
		}
	}

	/// Given a Model, create an OpenglRenderer:
	/// - Setup buffers and shaders.
	/// - Decode textures.
	/// - Upload static buffer data and textures.
	pub fn new(gl: glow::Context, model: &Model) -> Result<Self, OpenglRendererError> {
		// Initialize framebuffers
		let composite_framebuffer;
		let cf_albedo;
		let cf_emissive;
		let cf_bump;
		let cf_stencil;
		unsafe {
			cf_albedo = gl.create_texture().map_err(OpenglRendererError::Opengl)?;
			cf_emissive = gl.create_texture().map_err(OpenglRendererError::Opengl)?;
			cf_bump = gl.create_texture().map_err(OpenglRendererError::Opengl)?;
			cf_stencil = gl.create_texture().map_err(OpenglRendererError::Opengl)?;

			composite_framebuffer = gl.create_framebuffer().map_err(OpenglRendererError::Opengl)?;
		}

		// Shaders
		let part_shader = PartShader::new(&gl)?;
		let part_mask_shader = PartMaskShader::new(&gl)?;
		let composite_shader = CompositeShader::new(&gl)?;
		let composite_mask_shader = CompositeMaskShader::new(&gl)?;

		let support_debug_extension = gl.supported_extensions().contains("GL_KHR_debug");

		let inox_buffers = model
			.puppet
			.render_ctx
			.as_ref()
			.expect("Rendering for a puppet must be initialized before creating a renderer.");
		let vao = setup_gl_buffers(
			&gl,
			inox_buffers.vertex_buffers.verts.as_slice(),
			inox_buffers.vertex_buffers.uvs.as_slice(),
			inox_buffers.vertex_buffers.deforms.as_slice(),
			inox_buffers.vertex_buffers.indices.as_slice(),
		)?;

		// decode textures in parallel
		let shalltexs = decode_model_textures(model.textures.iter());
		let textures = shalltexs
			.iter()
			.enumerate()
			.map(|e| {
				tracing::debug!("Uploading shallow texture {:?}", e.0);
				texture::Texture::from_shallow_texture(&gl, e.1).map_err(|e| OpenglRendererError::Opengl(e.to_string()))
			})
			.collect::<Result<Vec<_>, _>>()?;

		let renderer = Self {
			gl,
			support_debug_extension,
			camera: Camera::default(),
			viewport: UVec2::default(),
			cache: RefCell::new(GlCache::default()),

			vao,

			composite_framebuffer,
			cf_albedo,
			cf_emissive,
			cf_bump,
			cf_stencil,

			part_shader,
			part_mask_shader,
			composite_shader,
			composite_mask_shader,

			textures,
		};

		// Set emission strength once (it doesn't change anywhere else)
		renderer.bind_shader(&renderer.part_shader);
		renderer.part_shader.set_emission_strength(&renderer.gl, 1.);

		Ok(renderer)
	}
}

impl InoxRenderer for OpenglRenderer {
	fn on_begin_masks(&self, masks: &Masks) {
		let gl = &self.gl;

		unsafe {
			gl.enable(glow::STENCIL_TEST);
			gl.clear_stencil(!masks.has_masks() as i32);
			gl.clear(glow::STENCIL_BUFFER_BIT);

			gl.color_mask(false, false, false, false);
			gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
			gl.stencil_mask(0xff);
		}

		let part_mask_shader = &self.part_mask_shader;
		self.bind_shader(part_mask_shader);
		part_mask_shader.set_threshold(gl, masks.threshold.clamp(0.0, 1.0));
	}

	fn on_begin_mask(&self, mask: &Mask) {
		let gl = &self.gl;
		unsafe {
			gl.stencil_func(glow::ALWAYS, (mask.mode == MaskMode::Mask) as i32, 0xff);
		}
	}

	fn on_begin_masked_content(&self) {
		let gl = &self.gl;
		unsafe {
			gl.stencil_func(glow::EQUAL, 1, 0xff);
			gl.stencil_mask(0x00);

			gl.color_mask(true, true, true, true);
		}
	}

	fn on_end_mask(&self) {
		let gl = &self.gl;
		unsafe {
			gl.stencil_mask(0xff);
			gl.stencil_func(glow::ALWAYS, 1, 0xff);
			gl.disable(glow::STENCIL_TEST);
		}
	}

	fn draw_textured_mesh_content(
		&self,
		as_mask: bool,
		components: &TexturedMeshComponents,
		render_ctx: &TexturedMeshRenderCtx,
		_id: InoxNodeUuid,
	) {
		let gl = &self.gl;

		// TODO: plain masks, meshes as masks without textures
		/*
		maskShader.use();
		maskShader.setUniform(offset, data.origin);
		maskShader.setUniform(mvp, inGetCamera().matrix * transform.matrix());

		// Enable points array
		glEnableVertexAttribArray(0);
		glBindBuffer(GL_ARRAY_BUFFER, vbo);
		glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 0, null);

		// Bind index buffer
		this.bindIndex();

		// Disable the vertex attribs after use
		glDisableVertexAttribArray(0);
		*/

		self.bind_part_textures(components.data);
		self.set_blend_mode(components.drawable.blending.mode);

		let mvp = self.camera.matrix(self.viewport.as_vec2()) * *components.transform;

		if as_mask {
			// if as_mask is set, in .on_begin_masks():
			// - part_mask_shader must have been bound and prepared.
			// - mask threshold must have been uploaded.

			// vert uniforms
			self.part_mask_shader.set_mvp(gl, mvp);
		} else {
			let part_shader = &self.part_shader;
			self.bind_shader(part_shader);

			// vert uniforms
			part_shader.set_mvp(gl, mvp);

			// frag uniforms
			part_shader.set_opacity(gl, components.drawable.blending.opacity);
			part_shader.set_mult_color(gl, components.drawable.blending.tint);
			part_shader.set_screen_color(gl, components.drawable.blending.screen_tint);
		}

		unsafe {
			gl.draw_elements(
				glow::TRIANGLES,
				render_ctx.index_len as i32,
				glow::UNSIGNED_SHORT,
				render_ctx.index_offset as i32 * mem::size_of::<u16>() as i32,
			);
		}
	}

	fn begin_composite_content(
		&self,
		_as_mask: bool,
		_components: &CompositeComponents,
		_render_ctx: &CompositeRenderCtx,
		_id: InoxNodeUuid,
	) {
		self.clear_texture_cache();

		let gl = &self.gl;
		unsafe {
			gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.composite_framebuffer));
			gl.disable(glow::DEPTH_TEST);
			gl.draw_buffers(&[
				glow::COLOR_ATTACHMENT0,
				glow::COLOR_ATTACHMENT1,
				glow::COLOR_ATTACHMENT2,
			]);
			gl.clear_color(0.0, 0.0, 0.0, 0.0);
			gl.clear(glow::COLOR_BUFFER_BIT);

			// Everything else is the actual texture used by the meshes at id 0
			gl.active_texture(glow::TEXTURE0);
			gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
		}
	}

	fn finish_composite_content(
		&self,
		as_mask: bool,
		components: &CompositeComponents,
		_render_ctx: &CompositeRenderCtx,
		_id: InoxNodeUuid,
	) {
		let gl = &self.gl;

		self.clear_texture_cache();
		unsafe {
			gl.bind_framebuffer(glow::FRAMEBUFFER, None);
		}

		let blending = &components.drawable.blending;
		if as_mask {
			/*
			cShaderMask.use();
			cShaderMask.setUniform(mopacity, opacity);
			cShaderMask.setUniform(mthreshold, threshold);
			glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA);
			*/
			todo!()
		} else {
			unsafe {
				gl.bind_vertex_array(Some(self.vao));

				gl.active_texture(glow::TEXTURE0);
				gl.bind_texture(glow::TEXTURE_2D, Some(self.cf_albedo));
				gl.active_texture(glow::TEXTURE1);
				gl.bind_texture(glow::TEXTURE_2D, Some(self.cf_emissive));
				gl.active_texture(glow::TEXTURE2);
				gl.bind_texture(glow::TEXTURE_2D, Some(self.cf_bump));
			}

			self.set_blend_mode(blending.mode);

			let opacity = blending.opacity.clamp(0.0, 1.0);
			let tint = blending.tint.clamp(Vec3::ZERO, Vec3::ONE);
			let screen_tint = blending.screen_tint.clamp(Vec3::ZERO, Vec3::ONE);

			let composite_shader = &self.composite_shader;
			self.bind_shader(composite_shader);
			composite_shader.set_opacity(gl, opacity);
			composite_shader.set_mult_color(gl, tint);
			composite_shader.set_screen_color(gl, screen_tint);
		}

		unsafe {
			gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_SHORT, 0);
		}
	}
}

impl OpenglRenderer {
	/// Update the renderer with latest puppet data.
	pub fn on_begin_draw(&self, puppet: &Puppet) {
		let gl = &self.gl;

		// TODO: calculate this matrix only once per draw pass.
		// let matrix = self.camera.matrix(self.viewport.as_vec2());

		unsafe {
			gl.bind_vertex_array(Some(self.vao));
			upload_deforms_to_gl(
				gl,
				puppet
					.render_ctx
					.as_ref()
					.expect("Rendering for a puppet must be initialized by now.")
					.vertex_buffers
					.deforms
					.as_slice(),
			);
			gl.enable(glow::BLEND);
			gl.disable(glow::DEPTH_TEST);
		}
	}

	/// Renderer cleaning up after one frame.
	pub fn on_end_draw(&self, _puppet: &Puppet) {
		let gl = &self.gl;

		unsafe {
			gl.bind_vertex_array(None);
		}
	}
}
