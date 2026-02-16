use inox2d::model::Model;
use wgpu;

mod pipeline;
mod shader;
mod shaders;

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

	part_pipeline: pipeline::Pipeline<basic_vert::Shader, basic_frag::Shader>,
	part_mask_pipeline: pipeline::Pipeline<basic_vert::Shader, basic_mask_frag::Shader>,

	composite_pipeline: pipeline::Pipeline<composite_vert::Shader, composite_frag::Shader>,
	composite_mask_pipeline: pipeline::Pipeline<composite_vert::Shader, composite_mask_frag::Shader>,
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
			part_pipeline,
			part_mask_pipeline,
			composite_pipeline,
			composite_mask_pipeline,
		})
	}
}
