// Copyright (c) 2022 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::texture::Texture;

use glow::HasContext;

/// Uploads a texture to OpenGL.
///
/// # Panics
///
/// Panics if OpenGL cannot create the texture.
///
/// # Safety
///
/// Make sure the bytes in `data` have the correct `width`, `height` and `format`.
pub(crate) unsafe fn upload_texture(
    gl: &glow::Context,
    width: u32,
    height: u32,
    data: Option<&[u8]>,
) -> glow::NativeTexture {
    let texture = gl.create_texture().unwrap();
    gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA as i32,
        width as i32,
        height as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        data,
    );
    texture
}

/// Loads a texture from memory and uploads it to the GPU.
pub(crate) fn load_texture(gl: &glow::Context, tex: &Texture) -> glow::NativeTexture {
    match tex {
        Texture::Rgba {
            width,
            height,
            data,
        } => unsafe { upload_texture(gl, *width, *height, Some(data)) },
    }
}
