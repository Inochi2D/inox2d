use wgpu::{Device, Queue, Texture};
/// Loads a TGA texture from memory and uploads it to the GPU.
///
/// # Panics
///
/// Panics if it couldn't read the texture.
pub(crate) fn load_texture(device: &Device, queue: &Queue, tex: &[u8]) -> Texture {
    let image = image::load_from_memory_with_format(tex, image::ImageFormat::Tga).unwrap();

    let rgba = image.to_rgba8();

    use image::GenericImageView;
    let dimensions = image.dimensions();

    upload_texture(device, queue, dimensions.0, dimensions.1, tex)
}

pub(crate) fn upload_texture(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,

    data: &[u8],
) -> Texture {
    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        // All textures are stored as 3D, we represent our 2D texture
        // by setting depth to 1.
        size: texture_size,
        mip_level_count: 1, // We'll talk about this a little later
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Most images are stored using sRGB so we need to reflect that here.
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
        // COPY_DST means that we want to copy data to this texture
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
    });

    queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        // The actual pixel data
        &data,
        // The layout of the texture
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * width),
            rows_per_image: std::num::NonZeroU32::new(height),
        },
        texture_size,
    );

    texture
}
