use wgpu;

use inox2d::model::Model;
use inox2d::texture::ShallowTexture;

#[derive(Clone)]
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

	pub fn empty_render_target(
		device: &wgpu::Device,
		encoder: &mut wgpu::CommandEncoder,
		width: u32,
		height: u32,
		format: wgpu::TextureFormat,
	) -> Self {
		let size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};
		let device_texture = device.create_texture(&wgpu::TextureDescriptor {
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format,
			usage: wgpu::TextureUsages::TEXTURE_BINDING
				| wgpu::TextureUsages::RENDER_ATTACHMENT
				| wgpu::TextureUsages::COPY_DST,
			label: Some("GBuffer"),
			view_formats: &[],
		});

		let view = device_texture.create_view(&wgpu::TextureViewDescriptor::default());
		let empty = Self { device_texture, view };

		empty.clear(encoder);
		empty
	}

	// Clear the texture.
	pub fn clear(&self, encoder: &mut wgpu::CommandEncoder) {
		encoder.clear_texture(
			self.texture(),
			&wgpu::ImageSubresourceRange {
				aspect: wgpu::TextureAspect::All,
				base_mip_level: 0,
				mip_level_count: None,
				base_array_layer: 0,
				array_layer_count: None,
			},
		);
	}

	pub fn texture(&self) -> &wgpu::Texture {
		&self.device_texture
	}

	pub fn view(&self) -> &wgpu::TextureView {
		&self.view
	}

	pub fn as_color_attachment(&self) -> wgpu::RenderPassColorAttachment<'_> {
		wgpu::RenderPassColorAttachment {
			view: &self.view,
			resolve_target: None,
			depth_slice: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			},
		}
	}
}

pub struct DepthStencilBuffer {
	device_texture: wgpu::Texture,
	view: wgpu::TextureView,
	format: wgpu::TextureFormat,
}

impl DepthStencilBuffer {
	pub fn empty_render_target(
		device: &wgpu::Device,
		encoder: &mut wgpu::CommandEncoder,
		width: u32,
		height: u32,
		format: wgpu::TextureFormat,
	) -> Self {
		let size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};
		let device_texture = device.create_texture(&wgpu::TextureDescriptor {
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format,
			usage: wgpu::TextureUsages::TEXTURE_BINDING
				| wgpu::TextureUsages::RENDER_ATTACHMENT
				| wgpu::TextureUsages::COPY_DST,
			label: Some("GBuffer"),
			view_formats: &[],
		});

		let view = device_texture.create_view(&wgpu::TextureViewDescriptor::default());
		let empty = Self {
			device_texture,
			view,
			format,
		};

		empty.clear(encoder);
		empty
	}

	// Clear the texture.
	pub fn clear(&self, encoder: &mut wgpu::CommandEncoder) {
		encoder.clear_texture(
			self.texture(),
			&wgpu::ImageSubresourceRange {
				aspect: wgpu::TextureAspect::All,
				base_mip_level: 0,
				mip_level_count: None,
				base_array_layer: 0,
				array_layer_count: None,
			},
		);
	}

	pub fn texture(&self) -> &wgpu::Texture {
		&self.device_texture
	}

	pub fn view(&self) -> &wgpu::TextureView {
		&self.view
	}

	pub fn format(&self) -> wgpu::TextureFormat {
		self.format
	}

	pub fn as_depth_stencil_attachment_rw(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
		wgpu::RenderPassDepthStencilAttachment {
			view: &self.view,
			depth_ops: None,
			stencil_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Store,
			}),
		}
	}

	pub fn as_depth_stencil_attachment_ro(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
		wgpu::RenderPassDepthStencilAttachment {
			view: &self.view,
			depth_ops: None,
			stencil_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Load,
				store: wgpu::StoreOp::Discard,
			}),
		}
	}

	pub fn as_depth_stencil_attachment_clear(&self, clear_value: u32) -> wgpu::RenderPassDepthStencilAttachment<'_> {
		wgpu::RenderPassDepthStencilAttachment {
			view: &self.view,
			depth_ops: None,
			stencil_ops: Some(wgpu::Operations {
				load: wgpu::LoadOp::Clear(clear_value),
				store: wgpu::StoreOp::Store,
			}),
		}
	}
}

/// Structure that holds render targets for interim rendering results.
pub struct GBuffer {
	albedo: DeviceTexture,
	emissive: DeviceTexture,
	bump: DeviceTexture,
	stencil: DepthStencilBuffer,
}

impl GBuffer {
	pub fn new(
		device: &wgpu::Device,
		encoder: &mut wgpu::CommandEncoder,
		width: u32,
		height: u32,
		format: wgpu::TextureFormat,
		depth_format: wgpu::TextureFormat,
	) -> Self {
		Self {
			albedo: DeviceTexture::empty_render_target(device, encoder, width, height, wgpu::TextureFormat::Rgba8Uint),
			emissive: DeviceTexture::empty_render_target(device, encoder, width, height, format),
			bump: DeviceTexture::empty_render_target(device, encoder, width, height, wgpu::TextureFormat::Rgba8Uint),
			stencil: DepthStencilBuffer::empty_render_target(device, encoder, width, height, depth_format),
		}
	}

	pub fn albedo(&self) -> &DeviceTexture {
		&self.albedo
	}

	pub fn emissive(&self) -> &DeviceTexture {
		&self.emissive
	}

	pub fn bump(&self) -> &DeviceTexture {
		&self.bump
	}

	pub fn stencil(&self) -> &DepthStencilBuffer {
		&self.stencil
	}

	pub fn clear(&self, encoder: &mut wgpu::CommandEncoder) {
		self.albedo().clear(encoder);
		self.emissive().clear(encoder);
		self.bump().clear(encoder);
		self.stencil().clear(encoder);
	}

	pub fn as_color_attachments(&self) -> [Option<wgpu::RenderPassColorAttachment<'_>>; 3] {
		[
			Some(self.albedo.as_color_attachment()),
			Some(self.emissive.as_color_attachment()),
			Some(self.bump.as_color_attachment()),
		]
	}
}
