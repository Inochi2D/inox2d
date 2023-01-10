use std::io::{Read, self};

pub const INOCHI2D_SPEC_VERSION: &str = "1.0-alpha";

#[inline]
fn read_n<R: Read, const N: usize>(data: &mut R) -> io::Result<[u8; N]> {
    let mut buf = [0_u8; N];
    data.read_exact(&mut buf)?;
    Ok(buf)
}

#[inline]
fn read_u8<R: Read>(data: &mut R) -> io::Result<u8> {
    let buf = read_n::<_, 1>(data)?;
    Ok(u8::from_ne_bytes(buf))
}

#[inline]
fn read_be_u32<R: Read>(data: &mut R) -> io::Result<u32> {
    let buf = read_n::<_, 4>(data)?;
    Ok(u32::from_be_bytes(buf))
}

#[inline]
fn read_le_u16<R: Read>(data: &mut R) -> io::Result<u16> {
    let buf = read_n::<_, 2>(data)?;
    Ok(u16::from_le_bytes(buf))
}

#[inline]
fn read_vec<R: Read>(data: &mut R, n: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0_u8; n];
    data.read_exact(&mut buf)?;
    Ok(buf)
}

pub mod formats;
pub mod math;
pub mod mesh;
pub mod model;
pub mod nodes;
pub mod puppet;
pub mod renderers;
pub mod texture;