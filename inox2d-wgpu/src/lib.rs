use glam::Mat4;
use inox2d::model::{Model, ModelTexture};
use inox2d::node::{InoxNodeUuid, components, drawables}; //hey wait a second that's just a u32 newtype! UUIDs are four of those!
use inox2d::render::{self, InoxRenderer};
use inox2d::texture::decode_model_textures;
use wgpu;

mod pipeline;
mod shader;
mod shaders;
mod texture;

use crate::texture::{DeviceTexture, GBuffer};
use shader::UniformBlock;
use shaders::basic::{basic_frag, basic_mask_frag, basic_vert, composite_frag, composite_mask_frag, composite_vert};

#[derive(Debug, thiserror::Error)]
#[error("Could not initialize wgpu renderer: {0}")]
pub enum WgpuRendererError {
	CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
	RequestAdapterError(#[from] wgpu::RequestAdapterError),
	RequestDeviceError(#[from] wgpu::RequestDeviceError),
}

pub struct WgpuRenderer<'window> {
	surface: wgpu::Surface<'window>,
	config: wgpu::SurfaceConfiguration,
	gbuffer: Option<GBuffer>,

	part_shader_vert: basic_vert::Shader,
	part_shader_frag: basic_frag::Shader,
	part_shader_mask_frag: basic_mask_frag::Shader,

	part_pipeline: pipeline::Pipeline<basic_vert::Shader, basic_frag::Shader>,
	part_mask_pipeline: pipeline::Pipeline<basic_vert::Shader, basic_mask_frag::Shader>,

	composite_shader_vert: composite_vert::Shader,
	composite_shader_frag: composite_frag::Shader,
	composite_shader_mask_frag: composite_mask_frag::Shader,

	composite_pipeline: pipeline::Pipeline<composite_vert::Shader, composite_frag::Shader>,
	composite_mask_pipeline: pipeline::Pipeline<composite_vert::Shader, composite_mask_frag::Shader>,

	encoder: Option<wgpu::CommandEncoder>,

	model_textures: Vec<DeviceTexture>,
	model_sampler: wgpu::Sampler,

	last_mask_threshold: f32,

	device: wgpu::Device,
	queue: wgpu::Queue,
}

impl<'window> WgpuRenderer<'window> {
	pub async fn new(
		target: impl Into<wgpu::SurfaceTarget<'window>>,
		model: &Model,
	) -> Result<Self, WgpuRendererError> {
		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
		let surface = instance.create_surface(target)?;
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				compatible_surface: Some(&surface),
				..Default::default()
			})
			.await?;
		let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await?;

		// Find a suitable surface configuration.
		let surface_caps = surface.get_capabilities(&adapter);
		let surface_format = surface_caps.formats[0]; //TODO: SRGB?
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,

			//TODO: We don't know the size of our surface at init time.
			width: 640,
			height: 480,
			present_mode: surface_caps.present_modes[0],
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 2,
		};

		// Compile all our shaders now.
		let part_shader_vert = basic_vert::Shader::new(&device);
		let part_shader_frag = basic_frag::Shader::new(&device);
		let part_shader_mask_frag = basic_mask_frag::Shader::new(&device);

		let part_pipeline = pipeline::Pipeline::new(&device, &part_shader_vert, &part_shader_frag);
		let part_mask_pipeline = pipeline::Pipeline::new(&device, &part_shader_vert, &part_shader_mask_frag);

		let composite_shader_vert = composite_vert::Shader::new(&device);
		let composite_shader_frag = composite_frag::Shader::new(&device);
		let composite_shader_mask_frag = composite_mask_frag::Shader::new(&device);

		let composite_pipeline = pipeline::Pipeline::new(&device, &composite_shader_vert, &composite_shader_frag);
		let composite_mask_pipeline =
			pipeline::Pipeline::new(&device, &composite_shader_vert, &composite_shader_mask_frag);

		//TODO: Upload model textures, verts, uvs, deforms, indicies
		let decoded_textures = decode_model_textures(model.textures.iter());
		let mut texture_handles = vec![];
		for (index, texture) in decoded_textures.iter().enumerate() {
			texture_handles.push(DeviceTexture::new_from_model(&device, &queue, model, index, texture));
		}

		let model_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToBorder,
			address_mode_v: wgpu::AddressMode::ClampToBorder,
			address_mode_w: wgpu::AddressMode::ClampToBorder,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			..Default::default()
		});

		// Flush all pending work.
		// In wgpu, texture uploads etc will only execute at submit time
		queue.submit([]);

		Ok(WgpuRenderer {
			surface,
			config,
			gbuffer: None,
			part_shader_vert,
			part_shader_frag,
			part_shader_mask_frag,
			part_pipeline,
			part_mask_pipeline,
			composite_shader_vert,
			composite_shader_frag,
			composite_shader_mask_frag,
			composite_pipeline,
			composite_mask_pipeline,
			encoder: None,
			model_textures: texture_handles,
			model_sampler,
			last_mask_threshold: 0.0,
			device,
			queue,
		})
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		if width > 0 && height > 0 {
			self.config.width = width;
			self.config.height = height;
			self.surface.configure(&self.device, &self.config);
			self.gbuffer = Some(GBuffer::new(&self.device, &self.queue, width, height));
		}
	}

	fn textures_for_part(&self, part: &components::TexturedMesh) -> (&DeviceTexture, &DeviceTexture, &DeviceTexture) {
		(
			&self.model_textures[part.tex_albedo.raw()],
			&self.model_textures[part.tex_bumpmap.raw()],
			&self.model_textures[part.tex_emissive.raw()],
		)
	}
}

impl<'window> InoxRenderer for WgpuRenderer<'window> {
	fn begin_render(&mut self) {
		if self.encoder.is_some() {
			panic!("Recursive rendering is not permitted.");
		}

		if self.gbuffer.is_none() {
			panic!("Buffer is not yet set up.");
		}

		self.encoder = Some(self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Inox2DWGPU"),
		}));
	}

	fn on_begin_masks(&mut self, masks: &components::Masks) {
		self.last_mask_threshold = masks.threshold.clamp(0.0, 1.0);

		//TODO: Enable stencilling on the render target.
	}

	fn on_begin_mask(&self, mask: &components::Mask) {
		unimplemented!()
	}

	fn on_begin_masked_content(&self) {
		unimplemented!()
	}

	fn on_end_mask(&self) {
		unimplemented!()
	}

	fn draw_textured_mesh_content(
		&mut self,
		as_mask: bool,
		components: &drawables::TexturedMeshComponents,
		render_ctx: &render::TexturedMeshRenderCtx,
		id: InoxNodeUuid,
	) {
		if let Some(gbuffer) = self.gbuffer.as_ref() {
			//NOTE: borrowck doesn't want us borrowing the encoder, so we .take() it instead.
			let mut encoder = self.encoder.take().expect("encoder");
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("WgpuRenderer::draw_textured_mesh_content"),
				color_attachments: &gbuffer.as_color_attachments(),
				depth_stencil_attachment: gbuffer.as_depth_stencil_attachment(),
				occlusion_query_set: None,
				timestamp_writes: None,
				multiview_mask: None,
			});

			let (albedo, bumpmap, emissive) = self.textures_for_part(components.texture);

			//TODO: set blend mode
			let uni_in_vert = basic_vert::Input {
				// TODO: there is no provision for the renderer to learn the
				// current camera/viewport matrix OpenGLRenderer just has a
				// pub parameter for it which is dumb.
				mvp: Mat4::IDENTITY.to_cols_array_2d(),
				offset: [0.0; 2],
			}
			.into_buffer(&self.device);

			if as_mask {
				let uni_in_frag = basic_mask_frag::Input {
					threshold: self.last_mask_threshold,
				}
				.into_buffer(&self.device);

				self.part_mask_pipeline.bind_frag(
					&mut render_pass,
					Some(&self.part_shader_mask_frag.bind(
						&self.device,
						albedo.view(),
						&self.model_sampler,
						&uni_in_frag,
					)),
				);
				self.part_mask_pipeline.bind_vertex(
					&mut render_pass,
					Some(&self.part_shader_vert.bind(&self.device, &uni_in_vert)),
				);

				render_pass.set_pipeline(self.part_mask_pipeline.pipeline());
			} else {
				//Regular parts
				let uni_in_frag = basic_frag::Input {
					opacity: components.drawable.blending.opacity,
					multColor: components.drawable.blending.tint.into(),
					screenColor: components.drawable.blending.screen_tint.into(),
					emissionStrength: 1.0, //NOTE: OpenGL never sets this.
				}
				.into_buffer(&self.device);

				self.part_pipeline.bind_frag(
					&mut render_pass,
					Some(&self.part_shader_frag.bind(
						&self.device,
						albedo.view(),
						bumpmap.view(),
						emissive.view(),
						&self.model_sampler,
						&uni_in_frag,
					)),
				);
				self.part_pipeline.bind_vertex(
					&mut render_pass,
					Some(&self.part_shader_vert.bind(&self.device, &uni_in_vert)),
				);

				render_pass.set_pipeline(self.part_pipeline.pipeline());
			}

			//TODO: Actual draw elements call

			drop(render_pass); //NOTE: borrowck also needs us to do this
			self.encoder = Some(encoder);
		}
	}

	fn begin_composite_content(
		&self,
		as_mask: bool,
		components: &drawables::CompositeComponents,
		render_ctx: &render::CompositeRenderCtx,
		id: InoxNodeUuid,
	) {
	}

	fn finish_composite_content(
		&self,
		as_mask: bool,
		components: &drawables::CompositeComponents,
		render_ctx: &render::CompositeRenderCtx,
		id: InoxNodeUuid,
	) {
	}

	fn end_render_and_flush(&mut self) {
		let end = self.encoder.take().expect("encoder").finish();
		self.queue.submit(std::iter::once(end));
	}
}
