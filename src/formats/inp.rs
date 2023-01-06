use std::io::{self, Read};
use std::str::Utf8Error;

use image::ImageFormat;

use crate::model::{Model, ModelTexture};

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

#[inline]
fn read_n<R: Read, const N: usize>(data: &mut R) -> Result<[u8; N], ParseInpError> {
    let mut buf = [0_u8; N];
    data.read_exact(&mut buf)?;
    Ok(buf)
}

#[inline]
fn read_u8<R: Read>(data: &mut R) -> Result<u8, ParseInpError> {
    let buf = read_n::<_, 1>(data)?;
    Ok(u8::from_ne_bytes(buf))
}

#[inline]
fn read_be_u32<R: Read>(data: &mut R) -> Result<u32, ParseInpError> {
    let buf = read_n::<_, 4>(data)?;
    Ok(u32::from_be_bytes(buf))
}

#[inline]
fn read_vec<R: Read>(data: &mut R, n: usize) -> Result<Vec<u8>, ParseInpError> {
    let mut buf = vec![0_u8; n];
    data.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn parse_inp<R: Read>(mut data: R) -> Result<Model, ParseInpError> {
    // check magic bytes
    let magic = read_n::<_, 8>(&mut data)?;
    if magic != MAGIC {
        return Err(ParseInpError::IncorrectMagic);
    }

    // parse json payload into puppet
    let json_length = read_be_u32(&mut data)? as usize;
    let payload = read_vec(&mut data, json_length)?;
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

    // TODO: read EXTended section

    Ok(Model { puppet, textures })
}
