use std::fs::File;
use std::io::{self, BufRead, Seek};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gl::types::{GLint, GLuint};
use image::{ColorType, ImageError, ImageFormat, ImageOutputFormat};
use lazy_static::lazy_static;
use thiserror::Error;

/// Texture filtering mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filtering {
    /// Try to smooth out textures.
    Linear,
    /// Try to preserve pixel edges.
    /// Due to texture sampling being float-based this is imprecise.
    Point,
}

/// Texture wrapping mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrapping {
    /// Clamp texture sampling to be within the texture.
    Clamp = gl::CLAMP_TO_BORDER,
    /// Wrap the texture in every direction indefinitely.
    Repeat = gl::REPEAT,
    /// Wrap the texture mirrored in every direction indefinitely.
    Mirror = gl::MIRRORED_REPEAT,
}

/// A texture which is not bound to an OpenGL context.
/// Used for texture atlassing.
#[derive(Clone, Debug)]
pub struct ShallowTexture {
    /// 8-bit RGBA color data.
    data: Vec<u8>,
    /// Width of texture.
    width: u32,
    /// Height of texture.
    height: u32,
    /// Amount of color channels.
    color_type: ColorType,
    /// Amount of channels to convert to when passed to OpenGL.
    conv_color_type: ColorType,
}

#[derive(Error, Debug)]
pub enum TextureLoadError {
    #[error("An IO error occurred while loading a texture: {0}")]
    IoError(#[from] io::Error),
    #[error("An image reading error occurred while loading a texture: {0}")]
    ImageError(#[from] ImageError),
}

#[derive(Error, Debug)]
pub enum TextureSaveError {
    #[error("An IO error occurred while saving a texture: {0}")]
    IoError(#[from] io::Error),
    #[error("An image reading error occurred while saving a texture: {0}")]
    ImageError(#[from] ImageError),
}

impl ShallowTexture {
    pub fn open<R: BufRead + Seek>(
        mut reader: R,
        format: ImageFormat,
        conv_channels: Option<ColorType>,
    ) -> Result<Self, TextureLoadError> {
        let mut fdata = Vec::new();
        reader.read_to_end(&mut fdata)?;
        let dimage = image::load(reader, format)?;

        // Copy data from the image for the ShallowTexture
        let data = dimage.as_bytes().to_vec();
        let width = dimage.width();
        let height = dimage.height();
        let channels = dimage.color();
        let conv_channels = conv_channels.unwrap_or(channels);

        Ok(Self::new(data, width, height, channels, conv_channels))
    }

    /// Load uncompressed texture from memory.
    pub fn new(
        data: Vec<u8>,
        width: u32,
        height: u32,
        channels: ColorType,
        conv_channels: ColorType,
    ) -> Self {
        Self {
            data,
            width,
            height,
            color_type: channels,
            conv_color_type: conv_channels,
        }
    }

    /// Save image to file on disk.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), TextureSaveError> {
        image::save_buffer(path, &self.data, self.width, self.height, self.color_type)?;
        Ok(())
    }
}

/// A texture. Only format supported is unsigned 8-bit RGBA.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    id: GLuint,
    width: i32,
    height: i32,

    in_color_mode: GLuint,
    out_color_mode: GLuint,
    channels: u8,
    color_type: ColorType,
}

impl From<ShallowTexture> for Texture {
    fn from(shallow: ShallowTexture) -> Self {
        Self::new(
            shallow.data,
            shallow.width as i32,
            shallow.height as i32,
            shallow.color_type.channel_count(),
            shallow.conv_color_type,
        )
    }
}

impl Texture {
    /// Create an empty texture.
    pub fn empty(width: i32, height: i32, color_type: ColorType) -> Self {
        let empty = vec![0; width as usize * height as usize * color_type.channel_count() as usize];
        Self::new(empty, width, height, color_type.channel_count(), color_type)
    }

    /// Create a new texture from specified data
    pub fn new(
        data: Vec<u8>,
        width: i32,
        height: i32,
        in_channels: u8,
        out_color_type: ColorType,
    ) -> Self {
        let in_color_mode = match in_channels {
            1 => gl::RED,
            2 => gl::RG,
            3 => gl::RGB,
            _ => gl::RGBA,
        };
        let out_color_mode = in_color_mode;

        // Generate OpenGL texture
        let mut id = 0;
        unsafe { gl::GenTextures(1, &mut id) };

        let mut texture = Self {
            id,
            width,
            height,
            in_color_mode,
            out_color_mode,
            channels: out_color_type.channel_count(),
            color_type: out_color_type,
        };
        texture.set_data(data);

        // Set default filtering and wrapping
        texture.set_filtering(Filtering::Linear);
        texture.set_wrapping(Wrapping::Clamp);
        texture.set_anisotropy(in_get_max_anisotropy() / 2.);

        texture
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.bind(0);
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                self.out_color_mode as GLint,
                self.width,
                self.height,
                0,
                self.in_color_mode,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
        }
        self.gen_mipmap();
    }

    pub fn set_filtering(&self, filtering: Filtering) {
        self.bind(0);

        let filtering = match filtering {
            Filtering::Linear => gl::LINEAR_MIPMAP_LINEAR,
            Filtering::Point => gl::NEAREST,
        } as GLint;
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filtering);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filtering)
        };
    }

    pub fn set_anisotropy(&self, value: f32) {
        self.bind(0);
        unsafe {
            gl::TexParameterf(
                gl::TEXTURE_2D,
                GL_TEXTURE_MAX_ANISOTROPY,
                value.clamp(1., in_get_max_anisotropy()),
            )
        };
    }

    pub fn set_wrapping(&self, wrapping: Wrapping) {
        self.bind(0);
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrapping as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrapping as i32);
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, [0.; 4].as_ptr());
        }
    }

    /// Bind this texture
    ///
    /// Notes:
    /// - In release mode, the unit value is clamped to 31 (The max OpenGL texture unit value).
    /// - In debug mode, unit values over 31 will assert.
    pub fn bind(&self, unit: u32) {
        assert!(unit <= 31, "Outside maximum OpenGL texture unit value");
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit.clamp(0, 31));
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn gen_mipmap(&self) {
        self.bind(0);
        unsafe { gl::GenerateMipmap(gl::TEXTURE_2D) };
    }

    pub fn set_data_region(
        &self,
        data: &[u8],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        channels: u8,
    ) {
        self.bind(0);

        // Make sure we don't try to change the texture in an out of bounds area.
        assert!(
            x >= 0 && x + width <= self.width,
            "x offset is out of bounds (xoffset={}, xbound={})",
            x + width,
            self.width
        );
        assert!(
            y >= 0 && y + height <= self.height,
            "y offset is out of bounds (yoffset={}, ybound={})",
            y + height,
            self.height
        );

        let in_channel_mode = match channels {
            1 => gl::RED,
            2 => gl::RG,
            3 => gl::RGB,
            _ => gl::RGBA,
        };

        // Update the texture
        unsafe {
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                width,
                height,
                in_channel_mode,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            )
        };

        self.gen_mipmap();
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), TextureSaveError> {
        let mut file = File::create(path)?;
        image::write_buffer_with_format(
            &mut file,
            &self.get_texture_data(true),
            self.width as u32,
            self.height as u32,
            self.color_type,
            ImageOutputFormat::Png,
        )?;
        Ok(())
    }

    pub fn get_texture_data(&self, unmultiply: bool) -> Vec<u8> {
        let mut buf = vec![0; self.width as usize * self.height as usize * self.channels as usize];
        self.bind(0);
        unsafe {
            gl::GetTexImage(
                gl::TEXTURE_2D,
                0,
                self.out_color_mode,
                gl::UNSIGNED_BYTE,
                buf.as_mut_ptr() as *mut _,
            )
        };
        if unmultiply && self.channels == 4 {
            in_tex_unpremultiply(&mut buf);
        }
        buf
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn color_mode(&self) -> GLuint {
        self.out_color_mode
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn center(&self) -> (i32, i32) {
        (self.width / 2, self.height / 2)
    }

    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}

/// These constants is not present in the gl crate for some reason??
const GL_TEXTURE_MAX_ANISOTROPY: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY: u32 = 0x84FF;

pub fn in_get_max_anisotropy() -> f32 {
    let mut max: f32 = 0.;
    unsafe { gl::GetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY, &mut max) }
    max
}

#[allow(clippy::identity_op)]
pub fn in_tex_premultiply(buf: &mut [u8]) {
    for i in 0..buf.len() / 4 {
        buf[i * 4 + 0] = (buf[i * 4 + 0] as u32 * buf[i * 4 + 3] as u32 / 255) as u8;
        buf[i * 4 + 1] = (buf[i * 4 + 1] as u32 * buf[i * 4 + 3] as u32 / 255) as u8;
        buf[i * 4 + 2] = (buf[i * 4 + 2] as u32 * buf[i * 4 + 3] as u32 / 255) as u8;
    }
}

#[allow(clippy::identity_op)]
pub fn in_tex_unpremultiply(buf: &mut [u8]) {
    for i in 0..buf.len() / 4 {
        if buf[i * 4 + 3] == 0 {
            continue;
        }

        buf[i * 4 + 0] = (buf[i * 4 + 0] as u32 * 255 / buf[i * 4 + 3] as u32) as u8;
        buf[i * 4 + 1] = (buf[i * 4 + 1] as u32 * 255 / buf[i * 4 + 3] as u32) as u8;
        buf[i * 4 + 2] = (buf[i * 4 + 2] as u32 * 255 / buf[i * 4 + 3] as u32) as u8;
    }
}

static STARTED: AtomicBool = AtomicBool::new(false);
lazy_static! {
    static ref TEXTURE_BINDINGS: Mutex<Vec<Texture>> = Mutex::new(Vec::new());
}

pub fn in_begin_texture_loading() {
    assert!(
        !STARTED.load(Ordering::Relaxed),
        "Texture loading pass already started!"
    );
    STARTED.store(true, Ordering::Release);
}

pub fn in_get_texture_from_id(id: u32) -> Option<Texture> {
    assert!(
        STARTED.load(Ordering::Relaxed),
        "Texture loading pass not started!"
    );
    let guard = TEXTURE_BINDINGS.lock().unwrap();
    guard.get(id as usize).map(Texture::clone)
}

pub fn in_get_latest_texture() -> Option<Texture> {
    assert!(
        STARTED.load(Ordering::Relaxed),
        "Texture loading pass not started!"
    );
    let guard = TEXTURE_BINDINGS.lock().unwrap();
    guard.last().map(Texture::clone)
}

pub fn in_add_texture_binary(data: ShallowTexture) {
    let texture = Texture::from(data);
    let mut guard = TEXTURE_BINDINGS.lock().unwrap();
    guard.push(texture);
}

pub fn in_end_texture_loading() {
    assert!(
        STARTED.load(Ordering::Relaxed),
        "Texture loading pass not started!"
    );
    STARTED.store(false, Ordering::Release);
    let mut guard = TEXTURE_BINDINGS.lock().unwrap();
    guard.clear();
}
