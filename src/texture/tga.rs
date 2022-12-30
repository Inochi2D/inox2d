use super::Texture;

#[inline]
fn copy_intersperce(src: &[u8], dst: &mut Vec<u8>) {
    dst.extend_from_slice(&src[..3]);
    dst.push(255);
}

// This function decodes to RGBA instead of RGB, by interspercing a 255 byte every three bytes.
fn decode_rle_24(mut rle: &[u8], vec: &mut Vec<u8>) {
    while rle.len() > 18 {
        let c = rle[0];
        rle = &rle[1..];
        if c & 0x80 != 0 {
            let c = (c & !0x80) + 1;
            let pixel = &rle[..3];
            rle = &rle[3..];
            for _ in 0..c {
                copy_intersperce(pixel, vec);
            }
        } else {
            let c = c + 1;
            for _ in 0..c {
                let pixel = &rle[..3];
                rle = &rle[3..];
                copy_intersperce(pixel, vec);
            }
        }
    }
}

fn decode_rle_32(mut rle: &[u8], out: &mut Vec<u8>) {
    while rle.len() > 16 {
        let c = rle[0];
        rle = &rle[1..];
        if c & 0x80 != 0 {
            let c = (c & !0x80) + 1;
            let pixel = &rle[..4];
            rle = &rle[4..];
            for _ in 0..c {
                out.extend_from_slice(pixel);
            }
        } else {
            let c = c + 1;
            let pixels = &rle[..4 * c as usize];
            rle = &rle[4 * c as usize..];
            out.extend_from_slice(pixels);
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
