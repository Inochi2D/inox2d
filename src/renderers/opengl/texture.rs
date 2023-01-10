use glow::HasContext;
use image::{ImageBuffer, ImageError, Rgba};

use crate::model::ModelTexture;
use crate::texture::tga::TgaDecodeError;

#[derive(thiserror::Error, Debug)]
pub enum TextureError {
    #[error("Could not create texture: {0}")]
    Create(String),
    #[error("Could not load image data for texture: {0}")]
    LoadData(#[from] ImageError),
    #[error("Could not load TGA texture: {0}")]
    LoadTga(#[from] TgaDecodeError)
}

pub struct Texture {
    tex: glow::NativeTexture,
    width: u32,
    height: u32,
    bpp: u32,
}

impl Texture {
    pub fn new(gl: &glow::Context, mtex: &ModelTexture) -> Result<Self, TextureError> {
        let img_buf = image::load_from_memory_with_format(&mtex.data, mtex.format)
            .map_err(TextureError::LoadData)?
            .into_rgba8();

        Self::from_image_buffer_rgba(gl, img_buf)
    }

    /// Makes an educated guess about the image format. TGA is not supported by this function.
    pub fn from_memory(gl: &glow::Context, img_buf: &[u8]) -> Result<Self, TextureError> {
        let img_buf = image::load_from_memory(img_buf)
            .map_err(TextureError::LoadData)?
            .into_rgba8();

        Self::from_image_buffer_rgba(gl, img_buf)
    }

    pub fn from_image_buffer_rgba(
        gl: &glow::Context,
        img_buf: ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> Result<Self, TextureError> {
        let pixels = img_buf.to_vec();
        let width = img_buf.width();
        let height = img_buf.height();

        Self::from_raw_pixels(gl, &pixels, width, height)
    }

    pub fn from_raw_pixels(
        gl: &glow::Context,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Self, TextureError> {
        let bpp = 8 * (pixels.len() / (width as usize * height as usize)) as u32;

        let tex = unsafe { gl.create_texture().map_err(TextureError::Create)? };
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
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
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(pixels),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Ok(Texture {
            tex,
            width,
            height,
            bpp,
        })
    }

    pub fn bind(&self, gl: &glow::Context) {
        self.bind_on(gl, 0);
    }

    pub fn bind_on(&self, gl: &glow::Context, slot: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe { gl.bind_texture(glow::TEXTURE_2D, None) };
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bpp(&self) -> u32 {
        self.bpp
    }
}
