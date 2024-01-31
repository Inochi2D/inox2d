use std::io;

use image::{ImageBuffer, ImageError, ImageFormat, Rgba};
use tracing::error;

use crate::model::ModelTexture;

use self::tga::{read_tga, TgaDecodeError, TgaImage};

pub mod tga;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureId(pub(crate) usize);

impl TextureId {
	pub fn raw(&self) -> usize {
		self.0
	}
}

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

#[derive(Debug, thiserror::Error)]
enum DecodeTextureError {
	#[error("Could not decode TGA texture")]
	TgaDecode(
		#[from]
		#[source]
		TgaDecodeError,
	),

	#[error("Could not decode texture")]
	ImageDecode(
		#[from]
		#[source]
		ImageError,
	),
}

fn decode_texture(mtex: ModelTexture) -> Result<ShallowTexture, DecodeTextureError> {
	if mtex.format == ImageFormat::Tga {
		let tga_texture = read_tga(&mut io::Cursor::new(&mtex.data))?;
		Ok(ShallowTexture::from(tga_texture))
	} else {
		let img_buf = image::load_from_memory_with_format(&mtex.data, mtex.format)?;
		Ok(ShallowTexture::from(img_buf.into_rgba8()))
	}
}

#[cfg(target_arch = "wasm32")]
pub fn decode_model_textures<'a>(
	model_textures: impl ExactSizeIterator<Item = &'a ModelTexture>,
) -> Vec<ShallowTexture> {
	(model_textures.cloned())
		.map(decode_texture)
		.inspect(|res| {
			if let Err(e) = res {
				tracing::error!("{}", e);
			}
		})
		.filter_map(Result::ok)
		.collect::<Vec<_>>()
}

/// Decodes model textures in parallel, using as many threads as we can use minus one.
#[cfg(not(target_arch = "wasm32"))]
pub fn decode_model_textures<'a>(
	model_textures: impl ExactSizeIterator<Item = &'a ModelTexture>,
) -> Vec<ShallowTexture> {
	use std::sync::mpsc;

	// get number of optimal threads from computer
	let mut num_threads = std::thread::available_parallelism().unwrap().get();

	// remove at least one thread to not torture the computer
	if num_threads > 1 {
		num_threads -= 1;
	}

	// do not use more threads than there are images
	if num_threads > model_textures.len() {
		num_threads = model_textures.len();
	}

	// use channels to get Rs back from thread computation
	let (tx_all, rx_all) = mpsc::channel();

	let mut pipes = Vec::with_capacity(num_threads);
	for th in 0..num_threads {
		// thread-local channel
		let (tx, rx) = mpsc::channel::<(usize, ModelTexture)>();

		let tx_all = tx_all.clone();
		std::thread::Builder::new()
			.name(format!("Image Decoder Thread ({})", th))
			.spawn(move || {
				// get textures from the thread-local channel, decode them, and send them to the global channel
				while let Ok((i, texture)) = rx.recv() {
					match decode_texture(texture) {
						Ok(decoded) => tx_all.send((i, decoded)).unwrap(),
						Err(e) => tracing::error!("{}", e),
					}
				}
			})
			.unwrap();

		pipes.push(tx);
	}

	let n_model_textures = model_textures.len();

	// distribute texture decoding on all threads we make available
	for ((i, texture), tx) in model_textures.enumerate().zip(pipes.iter().cycle()) {
		// REMINDER: the texture data is behind an arc, so it's not actually being cloned
		tx.send((i, texture.clone())).unwrap();
	}

	let mut decoded = rx_all.into_iter().take(n_model_textures).collect::<Vec<_>>();
	decoded.sort_by_key(|&(i, _)| i);

	decoded.into_iter().map(|(_, tex)| tex).collect()
}
