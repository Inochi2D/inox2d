// Copyright (c) 2022 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::Cursor;

use image::codecs::png::PngDecoder;
use image::{ColorType, DynamicImage, ImageBuffer, ImageDecoder};

use super::Texture;

/// Decodes a PNG image into an RGBA texture.
pub fn decode(png: &[u8]) -> Texture {
    let decoder = PngDecoder::new(Cursor::new(png)).unwrap();
    let (width, height) = decoder.dimensions();
    let color_type = decoder.color_type();

    let mut data = vec![0_u8; decoder.total_bytes() as usize];
    decoder.read_image(&mut data).unwrap();

    let data = match color_type {
        ColorType::Rgba8 => data,
        ColorType::Rgb8 => {
            let rgb = ImageBuffer::from_raw(width, height, data).unwrap();
            let dynamic = DynamicImage::ImageRgb8(rgb);
            let rgba = dynamic.into_rgba8();
            rgba.into_vec()
        }
        _ => panic!("Unknown color type {color_type:?}"),
    };

    Texture::Rgba {
        width,
        height,
        data,
    }
}
