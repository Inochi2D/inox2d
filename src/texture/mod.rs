use std::io;

pub mod tga;

#[derive(Clone, Debug)]
pub enum CompressedTexture {
    Png(Vec<u8>),
    Tga(Vec<u8>),
    Bc7(Vec<u8>),
}

impl CompressedTexture {
    pub fn decode(&self) -> Texture {
        match self {
            CompressedTexture::Png(data) => {
                use image::ImageDecoder;
                let cursor = io::Cursor::new(data);
                let decoder = image::codecs::png::PngDecoder::new(cursor).unwrap();
                let (width, height) = decoder.dimensions();
                let mut data = vec![0_u8; decoder.total_bytes() as usize];
                let color_type = decoder.color_type();
                decoder.read_image(&mut data).unwrap();
                let data = match color_type {
                    image::ColorType::Rgba8 => data,
                    image::ColorType::Rgb8 => {
                        let rgb = image::ImageBuffer::from_raw(width, height, data).unwrap();
                        let dynamic = image::DynamicImage::ImageRgb8(rgb);
                        let rgba = dynamic.into_rgba8();
                        rgba.into_vec()
                    }
                    _ => panic!("Unknown color type {color_type:?}"),
                };
                Texture::Rgba {
                    width,
                    height,
                    data,
                }
            }
            CompressedTexture::Tga(data) => {
                let (width, height, data) = tga::decode(data);
                Texture::Rgba {
                    width,
                    height,
                    data,
                }
            }
            CompressedTexture::Bc7(_) => todo!("BC7 is still unimplemented"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Texture {
    Rgba {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
}

impl Texture {
    pub fn encode(&self, format: image::ImageFormat) -> CompressedTexture {
        match self {
            Texture::Rgba {
                width,
                height,
                data,
            } => {
                let buf = Vec::new();
                let mut buf = std::io::Cursor::new(buf);
                image::write_buffer_with_format(
                    &mut buf,
                    data,
                    *width,
                    *height,
                    image::ColorType::Rgba8,
                    format,
                )
                .unwrap();
                match format {
                    image::ImageFormat::Png => CompressedTexture::Png(buf.into_inner()),
                    image::ImageFormat::Tga => CompressedTexture::Tga(buf.into_inner()),
                    _ => panic!("Unsupported format {format:?}"),
                }
            }
        }
    }
}