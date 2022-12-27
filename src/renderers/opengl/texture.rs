use crate::texture::{CompressedTexture, Texture};
use std::sync::mpsc;

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
        glow::RGBA as i32,
        width as i32,
        height as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        data,
    );
    texture
}

/// Loads a texture from memory and uploads it to the GPU.
pub(crate) fn load_texture(gl: &glow::Context, tex: &Texture) -> glow::NativeTexture {
    match tex {
        Texture::Rgba {
            width,
            height,
            data,
        } => unsafe { upload_texture(gl, *width, *height, Some(data)) },
    }
}

#[cfg(feature = "parallel-tex-dec")]
pub fn decode_textures(textures: &mut Vec<CompressedTexture>) -> mpsc::Receiver<(usize, Texture)> {
    let mut num_threads = std::thread::available_parallelism().unwrap().get();
    if num_threads > 1 {
        num_threads -= 1;
    }
    if num_threads > textures.len() {
        num_threads = textures.len();
    }

    let (tx2, rx2) = mpsc::channel();
    let mut pipes = Vec::with_capacity(num_threads);
    for _ in 0..num_threads {
        let (tx, rx) = mpsc::channel::<(usize, CompressedTexture)>();
        let tx2 = tx2.clone();
        std::thread::Builder::new()
            .name(String::from("Texture Decoder"))
            .spawn(move || {
                while let Ok((i, tex)) = rx.recv() {
                    let tex = tex.decode();
                    tx2.send((i, tex)).unwrap();
                }
            })
            .unwrap();
        pipes.push(tx);
    }

    for ((i, tex), tx) in textures.drain(..).enumerate().zip(pipes.iter().cycle()) {
        tx.send((i, tex)).unwrap();
    }

    rx2
}

#[cfg(not(feature = "parallel-tex-dec"))]
pub fn decode_textures(textures: &mut Vec<CompressedTexture>) -> mpsc::Receiver<(usize, Texture)> {
    let (tx, rx) = mpsc::channel();
    for (i, tex) in textures.drain(..).enumerate() {
        let tex = tex.decode();
        tx.send((i, tex)).unwrap();
    }
    rx
}
