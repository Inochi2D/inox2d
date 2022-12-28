use std::sync::mpsc;

pub mod png;
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
