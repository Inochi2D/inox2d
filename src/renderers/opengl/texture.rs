use glow::HasContext;

/// Uploads a texture to OpenGL.
///
/// # Panics
///
/// Panics if OpenGL cannot create the texture.
///
/// # Safety
///
/// Make sure the bytes in `data` have the correct `width`, `height` and `format`.
pub(crate) unsafe fn upload_texture(
    gl: &glow::Context,
    width: u32,
    height: u32,
    format: u32,
    data: Option<&[u8]>,
) -> glow::NativeTexture {
    let texture = gl.create_texture().unwrap();
    gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        format as i32,
        width as i32,
        height as i32,
        0,
        format,
        glow::UNSIGNED_BYTE,
        data,
    );
    texture
}


/// Loads a TGA texture from memory and uploads it to the GPU.
///
/// # Panics
///
/// Panics if it couldn't read the texture.
pub(crate) fn load_texture(gl: &glow::Context, tex: &[u8]) -> glow::NativeTexture {
    // TODO: accept a ModelTexture to support any format
    match image::load_from_memory_with_format(tex, image::ImageFormat::Tga).unwrap() {
        image::DynamicImage::ImageRgba8(ref image) => {
            let (width, height) = image.dimensions();
            unsafe { upload_texture(gl, width, height, glow::RGBA, Some(image)) }
        }
        image::DynamicImage::ImageRgb8(ref image) => {
            let (width, height) = image.dimensions();
            unsafe { upload_texture(gl, width, height, glow::RGB, Some(image)) }
        }
        image => todo!("Unsupported image: {:?}", image),
    }
}
