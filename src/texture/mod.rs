pub mod tga;
pub mod png;

#[derive(Clone, Debug)]
pub enum CompressedTexture {
    Png(Vec<u8>),
    Tga(Vec<u8>),
    Bc7(Vec<u8>),
}

impl CompressedTexture {
    pub fn decode(&self) -> Texture {
        match self {
            CompressedTexture::Png(png) => png::decode(png),
            CompressedTexture::Tga(tga) => tga::decode(tga),
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
