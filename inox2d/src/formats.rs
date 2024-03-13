pub mod inp;
mod json;
mod payload;

use glam::Vec2;

use std::io::{self, Read};
use std::slice;

pub use json::JsonError;

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
fn read_vec<R: Read>(data: &mut R, n: usize) -> io::Result<Vec<u8>> {
	let mut buf = vec![0_u8; n];
	data.read_exact(&mut buf)?;
	Ok(buf)
}

#[inline]
fn f32s_as_vec2s(vec: &[f32]) -> &'_ [Vec2] {
	// SAFETY: the length of the slice never trespasses outside of the array
	unsafe { slice::from_raw_parts(vec.as_ptr() as *const Vec2, vec.len() / 2) }
}
