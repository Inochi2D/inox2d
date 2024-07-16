mod gl_buffer;
mod shader;
mod shaders;
pub mod texture;

use std::cell::RefCell;
use std::mem;
use std::ops::Deref;

use gl_buffer::RenderCtxOpenglExt;
use glam::{uvec2, Mat4, UVec2, Vec2, Vec3};
use glow::HasContext;
use inox2d::texture::{decode_model_textures, TextureId};

use inox2d::math::camera::Camera;
use inox2d::model::{Model, ModelTexture};
use inox2d::node::data::{BlendMode, Composite, Part};
use inox2d::puppet::Puppet;
use inox2d::render::{InoxRenderer, InoxRendererCommon, NodeRenderCtx, PartRenderCtx};

use self::shader::ShaderCompileError;
use self::shaders::{CompositeMaskShader, CompositeShader, PartMaskShader, PartShader};
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
	pub viewport: Option<UVec2>,
	pub blend_mode: Option<BlendMode>,
	pub program: Option<glow::Program>,
	pub vao: Option<glow::VertexArray>,
	pub albedo: Option<TextureId>,
}

impl GlCache {
	pub fn clear(&mut self) {
		*self = Self::default();
	}

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

#[allow(unused)]
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
	deform_buffer: Option<glow::Buffer>,

	textures: Vec<Texture>,
}

// TODO: remove the #[allow(unused)]
#[allow(unused)]
impl OpenglRenderer {
	pub fn new(gl: glow::Context) -> Result<Self, OpenglRendererError> {
		let vao = unsafe { gl.create_vertex_array().map_err(OpenglRendererError::Opengl)? };

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

			textures: Vec::new(),
			deform_buffer: None,
		};

		// Set emission strength once (it doesn't change anywhere else)
		renderer.bind_shader(&renderer.part_shader);
		renderer.part_shader.set_emission_strength(&renderer.gl, 1.);

		Ok(renderer)
	}

	fn upload_model_textures(&mut self, model_textures: &[ModelTexture]) -> Result<(), TextureError> {
		// decode textures in parallel
		let shalltexs = decode_model_textures(model_textures.iter());

		// upload textures
		for (i, shalltex) in shalltexs.iter().enumerate() {
			tracing::debug!("Uploading shallow texture {:?}", i);
			let tex = texture::Texture::from_shallow_texture(&self.gl, shalltex)?;
			self.textures.push(tex);
		}

		Ok(())
	}

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

	fn bind_shader<S: Deref<Target = glow::Program>>(&self, shader: &S) {
		let program = **shader;
		if !self.cache.borrow_mut().update_program(program) {
			return;
		}

		unsafe { self.gl.use_program(Some(program)) };
	}

	fn bind_part_textures(&self, part: &Part) {
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
	pub fn clear_texture_cache(&self) {
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
}

impl InoxRenderer for OpenglRenderer {
	type Error = OpenglRendererError;

	fn prepare(&mut self, model: &Model) -> Result<(), Self::Error> {
		self.deform_buffer = Some(unsafe { model.puppet.render_ctx.setup_gl_buffers(&self.gl, self.vao)? });

		match self.upload_model_textures(&model.textures) {
			Ok(_) => Ok(()),
			Err(_) => Err(OpenglRendererError::Opengl("Texture Upload Error.".to_string())),
		}
	}

	fn resize(&mut self, w: u32, h: u32) {
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

	fn clear(&self) {
		self.cache.borrow_mut().clear();

		unsafe {
			self.gl.clear(glow::COLOR_BUFFER_BIT);
		}
	}

	/*
		These functions should be reworked together:
		setup_gl_buffers -> should set up in a way so that the draw functions draws into a texture
		on_begin/end_scene -> prepares and ends drawing to texture. also post-processing
		draw_scene -> actually makes things appear on a surface
	*/

	fn on_begin_scene(&self) {
		todo!()
	}

	fn render(&self, puppet: &Puppet) {
		let gl = &self.gl;
		unsafe {
			if let Some(deform_buffer) = self.deform_buffer {
				puppet.render_ctx.upload_deforms_to_gl(gl, deform_buffer);
			}

			gl.enable(glow::BLEND);
			gl.disable(glow::DEPTH_TEST);
		}

		let camera = self
			.camera
			.matrix(Vec2::new(self.viewport.x as f32, self.viewport.y as f32));
		self.draw(&camera, puppet);
	}

	fn on_end_scene(&self) {
		todo!()
	}

	fn draw_scene(&self) {
		todo!()
	}

	fn on_begin_mask(&self, has_mask: bool) {
		let gl = &self.gl;
		unsafe {
			gl.enable(glow::STENCIL_TEST);
			gl.clear_stencil(!has_mask as i32);
			gl.clear(glow::STENCIL_BUFFER_BIT);

			gl.color_mask(false, false, false, false);
			gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
			gl.stencil_mask(0xff);
		}
	}

	fn set_mask_mode(&self, dodge: bool) {
		let gl = &self.gl;
		unsafe {
			gl.stencil_func(glow::ALWAYS, !dodge as i32, 0xff);
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

	fn draw_mesh_self(&self, _as_mask: bool, _camera: &Mat4) {
		// TODO

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
		todo!()
	}

	fn draw_part_self(
		&self,
		as_mask: bool,
		camera: &Mat4,
		node_render_ctx: &NodeRenderCtx,
		part: &Part,
		part_render_ctx: &PartRenderCtx,
	) {
		let gl = &self.gl;

		self.bind_part_textures(part);
		self.set_blend_mode(part.draw_state.blend_mode);

		let part_shader = &self.part_shader;
		self.bind_shader(part_shader);
		// vert uniforms
		let mvp = *camera * node_render_ctx.trans;

		if as_mask {
			let part_mask_shader = &self.part_mask_shader;
			self.bind_shader(part_mask_shader);

			// vert uniforms
			part_mask_shader.set_mvp(gl, mvp);

			// frag uniforms
			part_mask_shader.set_threshold(gl, part.draw_state.mask_threshold.clamp(0.0, 1.0));
		} else {
			let part_shader = &self.part_shader;
			self.bind_shader(part_shader);

			// vert uniforms
			part_shader.set_mvp(gl, mvp);

			// frag uniforms
			part_shader.set_opacity(gl, part.draw_state.opacity);
			part_shader.set_mult_color(gl, part.draw_state.tint);
			part_shader.set_screen_color(gl, part.draw_state.screen_tint);
		}

		unsafe {
			gl.bind_vertex_array(Some(self.vao));
			gl.draw_elements(
				glow::TRIANGLES,
				part.mesh.indices.len() as i32,
				glow::UNSIGNED_SHORT,
				part_render_ctx.index_offset as i32 * mem::size_of::<u16>() as i32,
			);
		}
	}

	fn begin_composite_content(&self) {
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

	fn finish_composite_content(&self, as_mask: bool, composite: &Composite) {
		let gl = &self.gl;

		self.clear_texture_cache();
		unsafe {
			gl.bind_framebuffer(glow::FRAMEBUFFER, None);
		}

		let comp = &composite.draw_state;
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

			self.set_blend_mode(comp.blend_mode);

			let opacity = comp.opacity.clamp(0.0, 1.0);
			let tint = comp.tint.clamp(Vec3::ZERO, Vec3::ONE);
			let screen_tint = comp.screen_tint.clamp(Vec3::ZERO, Vec3::ONE);

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
