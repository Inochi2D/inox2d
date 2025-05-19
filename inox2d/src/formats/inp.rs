use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use std::sync::Arc;

use image::ImageFormat;

use crate::model::{Model, ModelTexture, VendorData};
use crate::puppet::Puppet;

use super::json::JsonError;
use super::payload::InoxParseError;
use super::{read_be_u32, read_n, read_u8, read_vec};

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

/// Parse `.inp` and `.inx` files.
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
	let puppet = Puppet::new_from_json(&payload)?;

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

		let data: Arc<[u8]> = read_vec(&mut data, tex_length)?.into();
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

/// Parse `.inp` and `.inx` files.
pub fn dump_inp<R: Read>(mut data: R, directory: &Path) -> Result<(), ParseInpError> {
	// check magic bytes
	let magic = read_n::<_, 8>(&mut data)?;
	if magic != MAGIC {
		return Err(ParseInpError::IncorrectMagic);
	}

	// parse json payload into puppet
	let length = read_be_u32(&mut data)? as usize;
	let payload = read_vec(&mut data, length)?;
	File::create(directory.join("payload.json"))?.write_all(&payload)?;

	// check texture section header
	let tex_sect = read_n::<_, 8>(&mut data).map_err(|_| ParseInpError::NoTexSect)?;
	if tex_sect != TEX_SECT {
		return Err(ParseInpError::NoTexSect);
	}

	// retrieve textures
	fs::create_dir(directory.join("textures"))?;
	let tex_count = read_be_u32(&mut data)? as usize;
	for i in 0..tex_count {
		let tex_length = read_be_u32(&mut data)? as usize;
		let tex_encoding = read_u8(&mut data)?;

		let format = match tex_encoding {
			0 => ImageFormat::Png, // PNG
			1 => ImageFormat::Tga, // TGA
			2 => return Err(ParseInpError::Bc7NotSupported),
			n => return Err(ParseInpError::InvalidTexEncoding(n)),
		};

		let extension = match format {
			ImageFormat::Png => "png",
			ImageFormat::Jpeg => "jpeg",
			ImageFormat::Gif => "gif",
			ImageFormat::WebP => "webp",
			ImageFormat::Pnm => "pnm",
			ImageFormat::Tiff => "tiff",
			ImageFormat::Tga => "tga",
			ImageFormat::Dds => "dds",
			ImageFormat::Bmp => "bmp",
			ImageFormat::Ico => "ico",
			ImageFormat::Hdr => "hdr",
			ImageFormat::OpenExr => "openexr",
			ImageFormat::Farbfeld => "farbfeld",
			ImageFormat::Avif => "avif",
			ImageFormat::Qoi => "qoi",
			_ => unreachable!(),
		};

		let data: Vec<u8> = read_vec(&mut data, tex_length)?;
		File::create(directory.join(format!("textures/tex-{:03}.{}", i, extension)))?.write_all(&data)?;
	}

	// read extended section header if present
	let Ok(ext_sect) = read_n::<_, 8>(&mut data) else {
		return Ok(());
	};

	fs::create_dir(directory.join("vendors"))?;
	if ext_sect == EXT_SECT {
		let ext_count = read_be_u32(&mut data)? as usize;

		for i in 0..ext_count {
			let length = read_be_u32(&mut data)? as usize;
			let name = read_vec(&mut data, length)?;
			let name = String::from_utf8(name)?;

			let length = read_be_u32(&mut data)? as usize;
			let payload = read_vec(&mut data, length)?;

			File::create(directory.join(format!("vendors/{:02} - {}.json", i, name)))?.write_all(&payload)?;
		}
	}

	Ok(())
}

pub fn dump_to_inp<W: Write>(directory: &Path, w: &mut W) -> io::Result<()> {
	let mut payload_file = File::open(directory.join("payload.json"))?;

	w.write_all(MAGIC)?;
	w.write_all(&(payload_file.metadata()?.len() as u32).to_be_bytes())?;
	io::copy(&mut payload_file, w)?;

	let mut texture_files = Vec::new();
	for tex_file in fs::read_dir(directory.join("textures"))? {
		let tex_file = tex_file?;
		let path = tex_file.path();
		let Some(ext) = path.extension() else {
			eprintln!("File {:?} has no extension, ignoring", tex_file.file_name());
			continue;
		};

		let Some(ext) = ext.to_str() else {
			eprintln!("File {:?} has unrecognized extension, ignoring", tex_file.file_name());
			continue;
		};

		let tex_encoding: u8 = match ext {
			"png" => 0,
			"tga" => 1,
			"bc7" => 2,
			ext => {
				eprintln!(
					"File {:?} has unsupported extension {:?}, ignoring",
					tex_file.file_name(),
					ext
				);
				continue;
			}
		};

		texture_files.push((tex_encoding, path));
	}

	w.write_all(TEX_SECT)?;
	w.write_all(&(texture_files.len() as u32).to_be_bytes())?;
	for (tex_encoding, tex_path) in texture_files {
		let mut tex_file = File::open(tex_path)?;

		let file_len = tex_file.metadata()?.len() as u32;
		println!("t {} | {:?} {} B", tex_encoding, file_len.to_be_bytes(), file_len);

		w.write_all(&file_len.to_be_bytes())?;
		w.write_all(&[tex_encoding])?;

		io::copy(&mut tex_file, w)?;
	}

	w.flush().unwrap();
	Ok(())
}
