use glow::HasContext;

unsafe fn upload_texture(
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
    gl.generate_mipmap(glow::TEXTURE_2D);
    texture
}

pub(super) fn load_texture(gl: &glow::Context, tex: &[u8]) -> glow::NativeTexture {
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
