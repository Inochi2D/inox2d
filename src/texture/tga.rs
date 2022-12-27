unsafe fn copy_intersperce(dst: *mut u8, src: *const u8) -> *mut u8 {
    dst.copy_from(src, 3);
    dst.offset(3).write(255);
    dst.offset(4)
}

// This function decodes to RGBA instead of RGB, by interspercing a 255 byte every three bytes.
fn decode_rle_24(mut rle: &[u8], mut ptr: *mut u8) {
    while rle.len() > 18 {
        let c = rle[0];
        rle = &rle[1..];
        if c & 0x80 != 0 {
            let c = (c & !0x80) + 1;
            let pixel = &rle[..3];
            rle = &rle[3..];
            for _ in 0..c {
                ptr = unsafe { copy_intersperce(ptr, pixel.as_ptr()) };
            }
        } else {
            let c = c + 1;
            for _ in 0..c {
                let pixel = &rle[..3];
                rle = &rle[3..];
                ptr = unsafe { copy_intersperce(ptr, pixel.as_ptr()) };
            }
        }
    }
}

fn decode_rle_32(mut rle: &[u8], mut ptr: *mut u8) {
    while rle.len() > 16 {
        let c = rle[0];
        rle = &rle[1..];
        if c & 0x80 != 0 {
            let c = (c & !0x80) + 1;
            let pixel = &rle[..4];
            rle = &rle[4..];
            for _ in 0..c {
                unsafe {
                    core::ptr::copy(pixel.as_ptr(), ptr, 4);
                    ptr = ptr.offset(4);
                }
            }
        } else {
            let c = c + 1;
            let pixels = &rle[..4 * c as usize];
            rle = &rle[4 * c as usize..];
            unsafe {
                core::ptr::copy(pixels.as_ptr(), ptr, 4 * c as usize);
                ptr = ptr.offset(4 * c as isize);
            }
        }
    }
}

pub fn decode(tga: &[u8]) -> (u32, u32, Vec<u8>) {
    let width = u16::from_le_bytes([tga[12], tga[13]]) as u32;
    let height = u16::from_le_bytes([tga[14], tga[15]]) as u32;
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    let pixel_depth = tga[16];
    match pixel_depth {
        24 => decode_rle_24(&tga[18..], data.as_mut_ptr()),
        32 => decode_rle_32(&tga[18..], data.as_mut_ptr()),
        depth => todo!("Unimplemented pixel depth {depth}"),
    }
    unsafe { data.set_len((width * height * 4) as usize) };
    (width, height, data)
}
