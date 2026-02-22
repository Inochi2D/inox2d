use glam::Mat4;
use inox2d::model::Model;
use inox2d::node::{InoxNodeUuid, components, drawables}; //hey wait a second that's just a u32 newtype! UUIDs are four of those!
use inox2d::render::{self, InoxRenderer};
use inox2d::texture::decode_model_textures;
use wgpu;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

mod pipeline;
mod shader;
mod shaders;
mod texture;

use crate::texture::{DeviceTexture, GBuffer};
use shader::UniformBlock;
use shaders::basic::{basic_frag, basic_mask_frag, basic_vert, composite_frag, composite_mask_frag, composite_vert};

/// Cast Vec2 to array.
///
/// SAFETY: This inherits the safety considerations of glam's own
/// `upload_array_to_gl`. Specifically, we rely on the fact that it's own Vec2
/// struct is plain-ol-data and we're only working with immutables.
///
/// NOTE: At some point, rewrite inox2D's vertex arrays to use bytemuck and a
/// custom Vec2 struct.
pub fn cast_vec2(array: &[glam::Vec2]) -> &[u8] {
	unsafe { std::slice::from_raw_parts(array.as_ptr() as *const u8, std::mem::size_of_val(array)) }
}

/// Cast u16s to array.
///
/// SAFETY: This inherits the safety considerations of glam's own
/// `upload_array_to_gl`.
///
/// NOTE: This probably can already be bytemucked
pub fn cast_index(array: &[u16]) -> &[u8] {
	unsafe { std::slice::from_raw_parts(array.as_ptr() as *const u8, std::mem::size_of_val(array)) }
}

#[derive(Debug, thiserror::Error)]
#[error("Could not initialize wgpu renderer: {0}")]
pub enum WgpuRendererError {
	CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
	RequestAdapterError(#[from] wgpu::RequestAdapterError),
	RequestDeviceError(#[from] wgpu::RequestDeviceError),

	#[error("Model rendering not initialized")]
	ModelRenderingNotInitialized,
}

pub struct WgpuRenderer<'window> {
	surface: wgpu::Surface<'window>,
	config: wgpu::SurfaceConfiguration,
	gbuffer: Option<GBuffer>,

	verts: wgpu::Buffer,
	uvs: wgpu::Buffer,
	deforms: wgpu::Buffer,
	indices: wgpu::Buffer,

	part_shader_vert: basic_vert::Shader,
	part_shader_frag: basic_frag::Shader,
	part_shader_mask_frag: basic_mask_frag::Shader,

	masked_depthstencil: wgpu::DepthStencilState,
	mask_depthstencil: wgpu::DepthStencilState,

	part_pipeline: pipeline::PipelineGroup<basic_vert::Shader, basic_frag::Shader>,
	part_mask_pipeline: pipeline::PipelineGroup<basic_vert::Shader, basic_mask_frag::Shader>,

	composite_shader_vert: composite_vert::Shader,
	composite_shader_frag: composite_frag::Shader,
	composite_shader_mask_frag: composite_mask_frag::Shader,

	composite_pipeline: pipeline::PipelineGroup<composite_vert::Shader, composite_frag::Shader>,
	composite_mask_pipeline: pipeline::PipelineGroup<composite_vert::Shader, composite_mask_frag::Shader>,

	encoder: Option<wgpu::CommandEncoder>,

	model_textures: Vec<DeviceTexture>,
	model_sampler: wgpu::Sampler,

	last_mask_threshold: f32,
	is_in_mask: bool,
	stencil_reference_value: u32,

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

		let masked_depthstencil = wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth24PlusStencil8,
			depth_write_enabled: false,
			depth_compare: wgpu::CompareFunction::Always,
			stencil: wgpu::StencilState {
				front: wgpu::StencilFaceState::IGNORE,
				back: wgpu::StencilFaceState::IGNORE,
				read_mask: 0xFF,
				write_mask: 0x00,
			},
			bias: wgpu::DepthBiasState::default(),
		};

		let mask_depthstencil = wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth24PlusStencil8,
			depth_write_enabled: false,
			depth_compare: wgpu::CompareFunction::Always,
			stencil: wgpu::StencilState {
				front: wgpu::StencilFaceState {
					compare: wgpu::CompareFunction::Always,
					fail_op: wgpu::StencilOperation::Keep,
					depth_fail_op: wgpu::StencilOperation::Keep,
					pass_op: wgpu::StencilOperation::Replace,
				},
				back: wgpu::StencilFaceState {
					compare: wgpu::CompareFunction::Always,
					fail_op: wgpu::StencilOperation::Keep,
					depth_fail_op: wgpu::StencilOperation::Keep,
					pass_op: wgpu::StencilOperation::Replace,
				},
				read_mask: 0xFF,
				write_mask: 0xFF,
			},
			bias: wgpu::DepthBiasState::default(),
		};

		//TODO: We need a pipeline per Inochi blending mode
		//(or some kind of ubershader blending)

		let part_pipeline = pipeline::PipelineGroup::new(part_shader_vert.clone(), part_shader_frag.clone());
		let part_mask_pipeline = pipeline::PipelineGroup::new(part_shader_vert.clone(), part_shader_mask_frag.clone());

		let composite_shader_vert = composite_vert::Shader::new(&device);
		let composite_shader_frag = composite_frag::Shader::new(&device);
		let composite_shader_mask_frag = composite_mask_frag::Shader::new(&device);

		let composite_pipeline =
			pipeline::PipelineGroup::new(composite_shader_vert.clone(), composite_shader_frag.clone());
		let composite_mask_pipeline =
			pipeline::PipelineGroup::new(composite_shader_vert.clone(), composite_shader_mask_frag.clone());

		let inox_buffers = model
			.puppet
			.render_ctx
			.as_ref()
			.ok_or(WgpuRendererError::ModelRenderingNotInitialized)?;
		//TODO: Change inox2d upstream to use a bytemuck-able array
		let verts = device.create_buffer_init(&BufferInitDescriptor {
			label: Some(&format!(
				"Inox2D {}::Verts",
				model.puppet.meta.name.as_deref().unwrap_or("<NAME NOT SPECIFIED>")
			)),
			contents: cast_vec2(inox_buffers.vertex_buffers.verts.as_slice()),
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		});
		let uvs = device.create_buffer_init(&BufferInitDescriptor {
			label: Some(&format!(
				"Inox2D {}::Verts",
				model.puppet.meta.name.as_deref().unwrap_or("<NAME NOT SPECIFIED>")
			)),
			contents: cast_vec2(inox_buffers.vertex_buffers.uvs.as_slice()),
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		});
		let deforms = device.create_buffer_init(&BufferInitDescriptor {
			label: Some(&format!(
				"Inox2D {}::Verts",
				model.puppet.meta.name.as_deref().unwrap_or("<NAME NOT SPECIFIED>")
			)),
			contents: cast_vec2(inox_buffers.vertex_buffers.deforms.as_slice()),
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		});
		let indices = device.create_buffer_init(&BufferInitDescriptor {
			label: Some(&format!(
				"Inox2D {}::Verts",
				model.puppet.meta.name.as_deref().unwrap_or("<NAME NOT SPECIFIED>")
			)),
			contents: cast_index(inox_buffers.vertex_buffers.indices.as_slice()),
			usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
		});

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
			verts,
			uvs,
			deforms,
			indices,
			part_shader_vert,
			part_shader_frag,
			part_shader_mask_frag,
			mask_depthstencil,
			masked_depthstencil,
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
			is_in_mask: false,
			stencil_reference_value: 1,
			device,
			queue,
		})
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		if width > 0 && height > 0 {
			self.config.width = width;
			self.config.height = height;
			self.surface.configure(&self.device, &self.config);
			self.gbuffer = Some(GBuffer::new(
				&self.device,
				&self.queue,
				width,
				height,
				wgpu::TextureFormat::Rgba32Float,
				wgpu::TextureFormat::Depth24PlusStencil8,
			));
		}
	}

	fn textures_for_part(&self, part: &components::TexturedMesh) -> (&DeviceTexture, &DeviceTexture, &DeviceTexture) {
		(
			&self.model_textures[part.tex_albedo.raw()],
			&self.model_textures[part.tex_bumpmap.raw()],
			&self.model_textures[part.tex_emissive.raw()],
		)
	}

	fn blend_mode_to_state(state: components::BlendMode) -> wgpu::BlendState {
		let component = match state {
			components::BlendMode::Normal => wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
				operation: wgpu::BlendOperation::Add,
			},
			components::BlendMode::Multiply => wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::Dst,
				dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
				operation: wgpu::BlendOperation::Add,
			},
			components::BlendMode::ColorDodge => wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::Dst,
				dst_factor: wgpu::BlendFactor::One,
				operation: wgpu::BlendOperation::Add,
			},
			components::BlendMode::LinearDodge => wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::One,
				operation: wgpu::BlendOperation::Add,
			},
			components::BlendMode::Screen => wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::OneMinusSrc,
				operation: wgpu::BlendOperation::Add,
			},
			components::BlendMode::ClipToLower => wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::DstAlpha,
				dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
				operation: wgpu::BlendOperation::Add,
			},
			components::BlendMode::SliceFromLower => wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
				dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
				operation: wgpu::BlendOperation::Subtract,
			},
		};

		wgpu::BlendState {
			color: component,
			alpha: component,
		}
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

		//TODO: read & translate OpenGLRenderer's `on_begin_draw` / `on_end_draw`
	}

	fn on_begin_masks(&mut self, masks: &components::Masks) {
		self.last_mask_threshold = masks.threshold.clamp(0.0, 1.0);

		//TODO: Erase the stencil buffer.
		//TODO: Enable stencilling on the render target.
	}

	fn on_begin_mask(&mut self, mask: &components::Mask) {
		self.stencil_reference_value = (mask.mode == components::MaskMode::Mask) as u32;
	}

	fn on_begin_masked_content(&mut self) {
		self.is_in_mask = true;
	}

	fn on_end_mask(&mut self) {
		self.is_in_mask = false;
	}

	fn draw_textured_mesh_content(
		&mut self,
		as_mask: bool,
		components: &drawables::TexturedMeshComponents,
		render_ctx: &render::TexturedMeshRenderCtx,
		_id: InoxNodeUuid,
	) {
		if let Some(gbuffer) = self.gbuffer.as_ref() {
			let depth_stencil_attachment = if as_mask {
				Some(gbuffer.stencil().as_depth_stencil_attachment_rw())
			} else if self.is_in_mask {
				Some(gbuffer.stencil().as_depth_stencil_attachment_ro())
			} else {
				None
			};

			//TODO: Do we even want blending on in Normal mode?
			let blend = Some(Self::blend_mode_to_state(components.drawable.blending.mode));

			//NOTE: borrowck doesn't want us borrowing the encoder, so we .take() it instead.
			let mut encoder = self.encoder.take().expect("encoder");
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("WgpuRenderer::draw_textured_mesh_content"),
				color_attachments: &gbuffer.as_color_attachments(),
				depth_stencil_attachment,
				occlusion_query_set: None,
				timestamp_writes: None,
				multiview_mask: None,
			});

			let (albedo, bumpmap, emissive) = self.textures_for_part(components.texture);
			let (albedo, bumpmap, emissive) = (albedo.clone(), bumpmap.clone(), emissive.clone());

			//TODO: set blend mode
			let uni_in_vert = basic_vert::Input {
				// TODO: there is no provision for the renderer to learn the
				// current camera/viewport matrix OpenGLRenderer just has a
				// pub parameter for it which is dumb.
				mvp: Mat4::IDENTITY.to_cols_array_2d(),
				offset: [0.0; 2],
			}
			.into_buffer(&self.device);

			render_pass.set_vertex_buffer(basic_vert::INPUT_LOCATION_VERTS, self.verts.slice(..));
			render_pass.set_vertex_buffer(basic_vert::INPUT_LOCATION_UVS, self.uvs.slice(..));
			render_pass.set_vertex_buffer(basic_vert::INPUT_LOCATION_DEFORM, self.deforms.slice(..));
			render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);

			if as_mask {
				//TODO: What happens if a mask is also masked?
				let pipeline = self.part_mask_pipeline.with_configuration(
					&self.device,
					[blend],
					[wgpu::ColorWrites::empty()],
					Some(self.mask_depthstencil.clone()),
				);
				let uni_in_frag = basic_mask_frag::Input {
					threshold: self.last_mask_threshold,
				}
				.into_buffer(&self.device);

				render_pass.set_pipeline(pipeline.pipeline());
				pipeline.bind_frag(
					&mut render_pass,
					Some(&self.part_shader_mask_frag.bind(
						&self.device,
						albedo.view(),
						&self.model_sampler,
						&uni_in_frag,
					)),
				);
				pipeline.bind_vertex(
					&mut render_pass,
					Some(&self.part_shader_vert.bind(&self.device, &uni_in_vert)),
				);

				render_pass.set_stencil_reference(self.stencil_reference_value);
			} else {
				let all = wgpu::ColorWrites::ALL;
				//Regular parts
				let pipeline = if self.is_in_mask {
					self.part_pipeline.with_configuration(
						&self.device,
						[blend, blend, blend],
						[all, all, all],
						Some(self.masked_depthstencil.clone()),
					)
				} else {
					self.part_pipeline
						.with_configuration(&self.device, [blend, blend, blend], [all, all, all], None)
				};

				let uni_in_frag = basic_frag::Input {
					opacity: components.drawable.blending.opacity,
					multColor: components.drawable.blending.tint.into(),
					screenColor: components.drawable.blending.screen_tint.into(),
					emissionStrength: 1.0, //NOTE: OpenGL never sets this.
				}
				.into_buffer(&self.device);

				render_pass.set_pipeline(pipeline.pipeline());
				pipeline.bind_frag(
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
				pipeline.bind_vertex(
					&mut render_pass,
					Some(&self.part_shader_vert.bind(&self.device, &uni_in_vert)),
				);

				render_pass.set_stencil_reference(1);
				render_pass.set_pipeline(pipeline.pipeline());
			}

			render_pass.draw_indexed(0..render_ctx.index_len as u32, render_ctx.index_offset as i32, 0..1);

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
