use wgpu;

use inox2d::model::Model;
use inox2d::texture::ShallowTexture;

pub struct DeviceTexture {
	device_texture: wgpu::Texture,
	view: wgpu::TextureView,
}

impl DeviceTexture {
	/// Submit a texture to be uploaded to the given WGPU device.
	///
	/// Note that the upload will not complete until the next queue submission.
	pub fn new_from_model(
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		model: &Model,
		index: usize,
		texture: &ShallowTexture,
	) -> Self {
		let size = wgpu::Extent3d {
			width: texture.width(),
			height: texture.height(),
			depth_or_array_layers: 1,
		};
		let device_texture = device.create_texture(&wgpu::TextureDescriptor {
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Uint, //TODO: SRGB?
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			label: Some(&format!(
				"Puppet texture: {}::{}",
				model.puppet.meta.name.as_deref().unwrap_or("<NAME NOT PROVIDED>"),
				index
			)),
			view_formats: &[],
		});

		queue.write_texture(
			wgpu::TexelCopyTextureInfo {
				texture: &device_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			texture.pixels(),
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(4 * texture.width()),
				rows_per_image: Some(texture.height()),
			},
			size,
		);

		let view = device_texture.create_view(&wgpu::TextureViewDescriptor::default());

		Self { device_texture, view }
	}

	pub fn texture(&self) -> &wgpu::Texture {
		&self.device_texture
	}

	pub fn view(&self) -> &wgpu::TextureView {
		&self.view
	}
}
