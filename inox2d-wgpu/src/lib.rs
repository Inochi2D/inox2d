use wgpu;
use inox2d::model::Model;

#[derive(Debug, thiserror::Error)]
#[error("Could not initialize wgpu renderer: {0}")]
pub enum WgpuRendererError {
	CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
	RequestAdapterError(#[from] wgpu::RequestAdapterError),
	RequestDeviceError(#[from] wgpu::RequestDeviceError),
}

pub struct WgpuRenderer<'window> {
	surface: wgpu::Surface<'window>,
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
		Ok(WgpuRenderer { surface })
	}
}
