// Copyright (c) 2022 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::Texture;

#[inline]
fn copy_intersperce(src: &[u8], dst: &mut Vec<u8>) {
    dst.extend_from_slice(&src[..3]);
    dst.push(255);
}

/// Decodes TGA data (24-bit BGR) to RGBA (interspercing a 255 byte every three bytes).
fn decode_rle_24(mut rle: &[u8], out: &mut Vec<u8>) {
    while rle.len() > 18 {
        let c = rle[0];
        rle = &rle[1..];

        if c & 0x80 != 0 {
            let c = (c & !0x80) + 1;
            let pixel = &rle[..3];

            rle = &rle[3..];
            for _ in 0..c {
                copy_intersperce(pixel, out);
            }
        } else {
            let c = c + 1;
            let pixels: Vec<u8> = rle[..3 * c as usize]
                .chunks_exact(3)
                .flat_map(|rle| [rle[2], rle[1], rle[0], 255])
                .collect();

            rle = &rle[3 * c as usize..];
            out.extend_from_slice(pixels.as_slice());
        }
    }
}

/// Decodes TGA data (32-bit BGRA) to RGBA.
fn decode_rle_32(mut rle: &[u8], out: &mut Vec<u8>) {
    while rle.len() > 16 {
        let c = rle[0];
        rle = &rle[1..];

        if c & 0x80 != 0 {
            let c = (c & !0x80) + 1;
            let pixel = &[rle[2], rle[1], rle[0], rle[3]];

            rle = &rle[4..];
            for _ in 0..c {
                out.extend_from_slice(pixel);
            }
        } else {
            let c = c + 1;
            let pixels: Vec<u8> = rle[..4 * c as usize]
                .chunks_exact(4)
                .flat_map(|rle| [rle[2], rle[1], rle[0], rle[3]])
                .collect();

            rle = &rle[4 * c as usize..];
            out.extend_from_slice(pixels.as_slice());
        }
    }
}

/// Decodes a TGA image into an RGBA texture.
pub fn decode(tga: &[u8]) -> Texture {
    let width = u16::from_le_bytes(tga[12..=13].try_into().unwrap()) as u32;
    let height = u16::from_le_bytes(tga[14..=15].try_into().unwrap()) as u32;
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    let pixel_depth = tga[16];
    match pixel_depth {
        24 => decode_rle_24(&tga[18..], &mut data),
        32 => decode_rle_32(&tga[18..], &mut data),
        depth => todo!("Unimplemented pixel depth {depth}"),
    }

    Texture::Rgba {
        width,
        height,
        data,
    }
}
