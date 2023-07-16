use std::io;

use image::{ImageBuffer, ImageFormat, Rgba};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tracing::error;

use crate::model::ModelTexture;

use self::tga::{read_tga, TgaImage};

pub mod tga;

pub struct ShallowTexture {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl ShallowTexture {
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl From<TgaImage> for ShallowTexture {
    fn from(value: TgaImage) -> Self {
        Self {
            pixels: value.data,
            width: value.header.width() as u32,
            height: value.header.height() as u32,
        }
    }
}

impl From<ImageBuffer<Rgba<u8>, Vec<u8>>> for ShallowTexture {
    fn from(value: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        Self {
            pixels: value.to_vec(),
            width: value.width(),
            height: value.height(),
        }
    }
}

pub fn decode_model_textures(model_textures: &[ModelTexture]) -> Vec<ShallowTexture> {
    model_textures
        .par_iter()
        .filter_map(|mtex| {
            if mtex.format == ImageFormat::Tga {
                match read_tga(&mut io::Cursor::new(&mtex.data)) {
                    Ok(img) => Some(ShallowTexture::from(img)),
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                }
            } else {
                let img_buf = image::load_from_memory_with_format(&mtex.data, mtex.format);

                match img_buf {
                    Ok(img_buf) => Some(ShallowTexture::from(img_buf.into_rgba8())),
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                }
            }
        })
        .collect::<Vec<_>>()
}
