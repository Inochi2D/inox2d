use std::io::{self, Read};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use image::ImageFormat;

use crate::model::{Model, ModelTexture, VendorData};
use crate::{read_be_u32, read_n, read_u8, read_vec};

use super::json::JsonError;
use super::serialize::{deserialize_puppet, InoxParseError};

#[derive(Debug, thiserror::Error)]
#[error("Could not parse INP file\n  - {0}")]
pub enum ParseInpError {
	#[error("magic bytes do not match, the file is not in the INP format")]
	IncorrectMagic,
	#[error("there is no texture section")]
	NoTexSect,
	#[error("BC7 texture encoding is not supported yet")]
	Bc7NotSupported,
	#[error("Invalid texture encoding: {0}")]
	InvalidTexEncoding(u8),
	Io(#[from] io::Error),
	Utf8(#[from] Utf8Error),
	FromUtf8(#[from] FromUtf8Error),
	JsonParse(#[from] json::Error),
	InoxParse(#[from] InoxParseError),
	Json(#[from] JsonError),
}

/// Trans rights!
const MAGIC: &[u8] = b"TRNSRTS\0";
/// Texture section header
const TEX_SECT: &[u8] = b"TEX_SECT";
/// Optional EXTended Vendor Data section for app provided settings for the puppet
const EXT_SECT: &[u8] = b"EXT_SECT";

pub fn parse_inp<R: Read>(mut data: R) -> Result<Model, ParseInpError> {
	// check magic bytes
	let magic = read_n::<_, 8>(&mut data)?;
	if magic != MAGIC {
		return Err(ParseInpError::IncorrectMagic);
	}

	// parse json payload into puppet
	let length = read_be_u32(&mut data)? as usize;
	let payload = read_vec(&mut data, length)?;
	let payload = std::str::from_utf8(&payload)?;
	let payload = json::parse(payload)?;
	let puppet = deserialize_puppet(&payload)?;

	// check texture section header
	let tex_sect = read_n::<_, 8>(&mut data).map_err(|_| ParseInpError::NoTexSect)?;
	if tex_sect != TEX_SECT {
		return Err(ParseInpError::NoTexSect);
	}

	// retrieve textures
	let tex_count = read_be_u32(&mut data)? as usize;
	let mut textures = Vec::with_capacity(tex_count);
	for _ in 0..tex_count {
		let tex_length = read_be_u32(&mut data)? as usize;
		let tex_encoding = read_u8(&mut data)?;
		let format = match tex_encoding {
			0 => ImageFormat::Png, // PNG
			1 => ImageFormat::Tga, // TGA
			2 => return Err(ParseInpError::Bc7NotSupported),
			n => return Err(ParseInpError::InvalidTexEncoding(n)),
		};
		let data = read_vec(&mut data, tex_length)?;
		textures.push(ModelTexture { format, data });
	}

	// read extended section header if present
	let vendors = match read_n::<_, 8>(&mut data) {
		Ok(ext_sect) if ext_sect == EXT_SECT => {
			let ext_count = read_be_u32(&mut data)? as usize;
			let mut vendors = Vec::with_capacity(ext_count);
			for _ in 0..ext_count {
				let length = read_be_u32(&mut data)? as usize;
				let name = read_vec(&mut data, length)?;
				let name = String::from_utf8(name)?;

				let length = read_be_u32(&mut data)? as usize;
				let payload = read_vec(&mut data, length)?;
				let payload = std::str::from_utf8(&payload)?;
				let payload = json::parse(payload)?;

				vendors.push(VendorData { name, payload });
			}
			vendors
		}
		_ => Vec::new(),
	};

	Ok(Model {
		puppet,
		textures,
		vendors,
	})
}
