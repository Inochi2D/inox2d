//! Port of imagefmt's TGA decoder written in D, licensed under BSD 2-Clause.
//!
//! See [LICENSE](https://github.com/tjhann/imagefmt/blob/master/LICENSE)
//!
//! https://github.com/tjhann/imagefmt

use std::io::{self, Read, Seek, Write};

use crate::{read_le_u16, read_u8};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DataType {
	NoData = 0,
	Idx = 1,
	TrueColor = 2,
	Gray = 3,
	IdxRle = 9,
	TruecolorRle = 10,
	GrayRle = 11,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid data type ({0})")]
pub struct InvalidDataType(u8);

impl TryFrom<u8> for DataType {
	type Error = InvalidDataType;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Ok(match value {
			0 => Self::NoData,
			1 => Self::Idx,
			2 => Self::TrueColor,
			3 => Self::Gray,
			9 => Self::IdxRle,
			10 => Self::TruecolorRle,
			11 => Self::GrayRle,
			n => return Err(InvalidDataType(n)),
		})
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TgaHeader {
	width: u16,
	height: u16,
	id_len: u8,
	palette_type: u8,
	data_type: DataType,
	bits_pp: u8,
	flags: u8,
}

impl TgaHeader {
	pub fn width(&self) -> u16 {
		self.width
	}

	pub fn height(&self) -> u16 {
		self.height
	}

	pub fn id_len(&self) -> u8 {
		self.id_len
	}

	pub fn palette_type(&self) -> u8 {
		self.palette_type
	}

	pub fn data_type(&self) -> DataType {
		self.data_type
	}

	pub fn bits_pp(&self) -> u8 {
		self.bits_pp
	}

	pub fn flags(&self) -> u8 {
		self.flags
	}
}

#[derive(Debug, thiserror::Error)]
pub enum TgaDecodeError {
	#[error("Couldn't decode TGA file: {0}")]
	Io(#[from] io::Error),
	#[error("Invalid TGA file header")]
	InvalidHeader,
	#[error("Unsupported TGA file: {0}")]
	Unsupported(&'static str),
	#[error("Image is too big")]
	TooBig,
}

// TGA doesn't have a signature so validate some values right here for detection.
pub(crate) fn read_tga_header<R: Read + Seek>(reader: &mut R) -> Result<TgaHeader, TgaDecodeError> {
	let id_len = read_u8(reader)?;
	let palette_type = read_u8(reader)?;
	let data_type = read_u8(reader)?.try_into().map_err(|_| TgaDecodeError::InvalidHeader)?;
	let palette_beg = read_le_u16(reader)?;
	let palette_len = read_le_u16(reader)?;
	let palette_bits = read_u8(reader)?;

	reader.seek(io::SeekFrom::Current(2 + 2))?; // origin (x, y)

	let width = read_le_u16(reader)?;
	let height = read_le_u16(reader)?;
	let bits_pp = read_u8(reader)?;
	let flags = read_u8(reader)?;

	if width < 1
		|| height < 1
		|| palette_type > 1
		|| (palette_type == 0 && (palette_beg > 0 || palette_len > 0 || palette_bits > 0))
	{
		return Err(TgaDecodeError::InvalidHeader);
	}

	Ok(TgaHeader {
		id_len,
		palette_type,
		data_type,
		width,
		height,
		bits_pp,
		flags,
	})
}

pub struct TgaImage {
	pub header: TgaHeader,
	pub data: Vec<u8>,
	pub channels: TgaChannels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TgaChannels {
	Y = 1,
	Ya = 2,
	Bgr = 3,
	Bgra = 4,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid TGA channel count ({0})")]
pub struct InvalidTgaChannelCount(u8);

impl TryFrom<u8> for TgaChannels {
	type Error = InvalidTgaChannelCount;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Ok(match value {
			1 => Self::Y,
			2 => Self::Ya,
			3 => Self::Bgr,
			4 => Self::Bgra,
			n => return Err(InvalidTgaChannelCount(n)),
		})
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitsPerChannel {
	B8,
	B16,
}

const TGA_FLAG_INTERLACED: u8 = 0xc0;
const TGA_FLAG_RIGHT_TO_LEFT: u8 = 0x10;
const TGA_FLAG_BITSPP: u8 = 0x0f;
const TGA_FLAG_ORIGIN_AT_TOP: u8 = 0x20;

const TGA_FLAG_PACKET_IS_RLE: u8 = 0x80;
const TGA_FLAG_PACKET_LEN: u8 = 0x7f;

const TGA_MAXIMUM_IMAGE_SIZE: u64 = 0x7fff_ffff;

/// Reads a TGA image into RGBA
pub fn read_tga<R: Read + Seek>(reader: &mut R) -> Result<TgaImage, TgaDecodeError> {
	let header = read_tga_header(reader)?;

	if header.flags & TGA_FLAG_INTERLACED > 0 {
		return Err(TgaDecodeError::Unsupported("interlaced"));
	}
	if header.flags & TGA_FLAG_RIGHT_TO_LEFT > 0 {
		return Err(TgaDecodeError::Unsupported("right-to-left"));
	}

	let attr_bits_pp = header.flags & TGA_FLAG_BITSPP;
	if attr_bits_pp != 0 && attr_bits_pp != 8 {
		// some set to 0 even if data has 8
		return Err(TgaDecodeError::Unsupported("bits per pixel != 8"));
	}
	if header.palette_type > 0 {
		return Err(TgaDecodeError::Unsupported("palette type != 0"));
	}

	match header.data_type {
		DataType::TrueColor | DataType::TruecolorRle => {
			if header.bits_pp != 24 && header.bits_pp != 32 {
				return Err(TgaDecodeError::Unsupported("bits per pixel != 24 or 32"));
			}
		}
		DataType::Gray | DataType::GrayRle => {
			if header.bits_pp != 8 && !(header.bits_pp == 16 && attr_bits_pp == 8) {
				return Err(TgaDecodeError::Unsupported("unsupported bits per pixel"));
			}
		}
		DataType::NoData => {
			return Err(TgaDecodeError::Unsupported("no data type"));
		}
		DataType::Idx => {
			return Err(TgaDecodeError::Unsupported("idx data type"));
		}
		DataType::IdxRle => {
			return Err(TgaDecodeError::Unsupported("idx rle data type"));
		}
	}

	let is_origin_at_top = header.flags & TGA_FLAG_ORIGIN_AT_TOP > 0;
	let is_rle = matches!(
		header.data_type,
		DataType::IdxRle | DataType::GrayRle | DataType::TruecolorRle
	);

	let channels: TgaChannels = (header.bits_pp / 8).try_into().unwrap(); // bytes per pixel
	let tchans = 4;
	let linebuf_size = header.width * channels as u16;
	let tline_size = header.width * tchans;

	let flip = !is_origin_at_top;
	let tstride = if flip { -(tline_size as i32) } else { tline_size as i32 };
	let mut ti = if flip {
		(header.height as isize - 1) * tline_size as isize
	} else {
		0
	} as usize;

	if header.width as u64 * header.height as u64 * tchans as u64 > TGA_MAXIMUM_IMAGE_SIZE {
		return Err(TgaDecodeError::TooBig);
	}

	let mut data = vec![0_u8; header.width as usize * header.height as usize * tchans as usize];
	let mut linebuf = vec![0_u8; linebuf_size as usize];

	if header.id_len > 0 {
		reader.seek(io::SeekFrom::Current(header.id_len as i64))?;
	}

	if !is_rle {
		for _ in 0..header.height {
			reader.read_exact(&mut linebuf)?;
			to_rgba(channels, &linebuf, &mut data[ti..ti + tline_size as usize])?;
			ti = ti.saturating_add_signed(tstride as isize);
		}
	} else {
		let mut pixel = [0_u8; 4];
		let mut packet_len = 0;
		let mut is_rle = false;

		for _ in 0..header.height {
			let mut wanted = linebuf_size as usize; // fill linebuf with unpacked data
			while wanted > 0 {
				if packet_len == 0 {
					let packet_head = read_u8(reader)?;
					is_rle = packet_head & TGA_FLAG_PACKET_IS_RLE > 0;
					packet_len = ((packet_head & TGA_FLAG_PACKET_LEN) + 1) as usize * channels as usize;
				}

				let gotten = linebuf_size as usize - wanted;
				let copy_size = wanted.min(packet_len);
				if is_rle {
					let channels = channels as usize;
					reader.read_exact(&mut pixel[..channels])?;

					let mut p = gotten;
					while p < gotten + copy_size {
						let mut place = &mut linebuf[p..p + channels];
						place.write_all(&pixel[..channels])?;
						p += channels;
					}
				} else {
					// raw packet
					reader.read_exact(&mut linebuf[gotten..gotten + copy_size])?;
				}

				wanted -= copy_size;
				packet_len -= copy_size;
			}

			to_rgba(channels, &linebuf, &mut data[ti..ti + tline_size as usize])?;
			ti = ti.saturating_add_signed(tstride as isize);
		}
	}

	Ok(TgaImage { header, data, channels })
}

fn to_rgba(channels: TgaChannels, src: &[u8], tgt: &mut [u8]) -> io::Result<()> {
	match channels {
		TgaChannels::Y => y_to_rgba(src, tgt),
		TgaChannels::Ya => ya_to_rgba(src, tgt),
		TgaChannels::Bgr => bgr_to_rgba(src, tgt),
		TgaChannels::Bgra => bgra_to_rgba(src, tgt),
	}
}

fn y_to_rgba(src: &[u8], tgt: &mut [u8]) -> io::Result<()> {
	for i in 0..src.len() {
		let (k, t) = (i, i * 4);
		let mut tgt = &mut tgt[t..t + 3];
		tgt.write_all(&[src[k]; 3])?;
		tgt[t + 3] = 255;
	}

	Ok(())
}

fn ya_to_rgba(src: &[u8], tgt: &mut [u8]) -> io::Result<()> {
	for i in 0..src.len() / 2 {
		let (k, t) = (i * 2, i * 4);
		let mut tgt = &mut tgt[t..t + 3];
		tgt.write_all(&[src[k]; 3])?;
		tgt[t + 3] = src[k + 1];
	}

	Ok(())
}

fn bgr_to_rgba(src: &[u8], tgt: &mut [u8]) -> io::Result<()> {
	for i in 0..src.len() / 3 {
		let (k, t) = (i * 3, i * 4);
		tgt[t] = src[k + 2];
		tgt[t + 1] = src[k + 1];
		tgt[t + 2] = src[k];
		tgt[t + 3] = 255;
	}

	Ok(())
}

fn bgra_to_rgba(src: &[u8], tgt: &mut [u8]) -> io::Result<()> {
	for i in 0..src.len() / 4 {
		let (k, t) = (i * 4, i * 4);
		tgt[t] = src[k + 2];
		tgt[t + 1] = src[k + 1];
		tgt[t + 2] = src[k];
		tgt[t + 3] = src[k + 3];
	}

	Ok(())
}
