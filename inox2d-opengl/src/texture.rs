use glow::HasContext;

use inox2d::texture::ShallowTexture;

#[derive(thiserror::Error, Debug)]
#[error("Could not create texture: {0}")]
pub struct TextureError(String);

pub struct Texture {
	tex: glow::Texture,
	width: u32,
	height: u32,
	bpp: u32,
}

impl Texture {
	pub fn from_shallow_texture(gl: &glow::Context, shalltex: &ShallowTexture) -> Result<Self, TextureError> {
		Self::from_raw_pixels(gl, shalltex.pixels(), shalltex.width(), shalltex.height())
	}

	pub fn from_raw_pixels(gl: &glow::Context, pixels: &[u8], width: u32, height: u32) -> Result<Self, TextureError> {
		let bpp = 8 * (pixels.len() / (width as usize * height as usize)) as u32;

		let tex = unsafe { gl.create_texture().map_err(TextureError)? };
		unsafe {
			gl.bind_texture(glow::TEXTURE_2D, Some(tex));
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_BORDER as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_BORDER as i32);
			gl.tex_parameter_f32_slice(glow::TEXTURE_2D, glow::TEXTURE_BORDER_COLOR, &[0.0; 4]);
			gl.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGBA8 as i32,
				width as i32,
				height as i32,
				0,
				glow::RGBA,
				glow::UNSIGNED_BYTE,
				Some(pixels),
			);
			gl.bind_texture(glow::TEXTURE_2D, None);
		}

		Ok(Texture {
			tex,
			width,
			height,
			bpp,
		})
	}

	pub fn bind(&self, gl: &glow::Context) {
		self.bind_on(gl, 0);
	}

	pub fn bind_on(&self, gl: &glow::Context, slot: u32) {
		unsafe {
			gl.active_texture(glow::TEXTURE0 + slot);
			gl.bind_texture(glow::TEXTURE_2D, Some(self.tex));
		}
	}

	pub fn unbind(&self, gl: &glow::Context) {
		unsafe { gl.bind_texture(glow::TEXTURE_2D, None) };
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn bpp(&self) -> u32 {
		self.bpp
	}
}

/// Uploads an empty texture.
///
/// # Safety
///
/// Make sure `ty` is a valid OpenGL number type
pub unsafe fn upload_empty(gl: &glow::Context, tex: glow::Texture, width: u32, height: u32, ty: u32) {
	let internal_format = if ty == glow::FLOAT { glow::RGBA32F } else { glow::RGBA8 } as i32;

	gl.bind_texture(glow::TEXTURE_2D, Some(tex));
	gl.tex_image_2d(
		glow::TEXTURE_2D,
		0,
		internal_format,
		width as i32,
		height as i32,
		0,
		glow::RGBA,
		ty,
		None,
	);
	gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
	gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
	gl.bind_texture(glow::TEXTURE_2D, None);
}
