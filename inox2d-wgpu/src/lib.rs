use glam::Mat4;
use inox2d::model::Model;
use inox2d::node::{InoxNodeUuid, components, drawables}; //hey wait a second that's just a u32 newtype! UUIDs are four of those!
use inox2d::render::{self, InoxRenderer};
use wgpu;

mod pipeline;
mod shader;
mod shaders;

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

		Ok(WgpuRenderer {
			surface,
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
			device,
			queue,
		})
	}
}

impl<'window> InoxRenderer for WgpuRenderer<'window> {
	fn begin_render(&mut self) {
		if self.encoder.is_some() {
			panic!("Recursive rendering is not permitted.");
		}

		self.encoder = Some(self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Inox2DWGPU"),
		}));
	}

	fn on_begin_masks(&self, masks: &components::Masks) {
		unimplemented!()
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
		let encoder = self.encoder.as_mut().expect("encoder");
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("WgpuRenderer::draw_textured_mesh_content"),
			color_attachments: &[],         //TODO: render target
			depth_stencil_attachment: None, //TODO: MASKS
			occlusion_query_set: None,
			timestamp_writes: None,
			multiview_mask: None,
		});

		if as_mask {
			render_pass.set_pipeline(self.part_mask_pipeline.pipeline());
			let uni_in = basic_vert::Input {
				// TODO: there is no provision for the renderer to learn the
				// current camera/viewport matrix OpenGLRenderer just has a
				// pub parameter for it which is dumb.
				mvp: Mat4::IDENTITY.to_cols_array_2d(),
				offset: [0.0; 2],
			}
			.into_buffer(&self.device);

			// TODO: We don't have good enough resource management to maintain
			// one uniform buffer per object, so we have to create and dispose
			// of them per frame.

			self.part_mask_pipeline.bind_vertex(
				&mut render_pass,
				Some(&self.part_shader_vert.bind(&self.device, &uni_in)),
			);
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
