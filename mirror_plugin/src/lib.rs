use std::ffi::{c_char, c_int, c_uchar};

// void process_image(
//      uint32_t width,
//      uint32_t height,
//      uint8_t* rgba_data,
//      const char* params
// );

const PIXEL_BYTES: u32 = 4;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut c_uchar,
    params: *const c_char,
) -> i32 {

    let Some(data_size) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|res| res.checked_mul(PIXEL_BYTES as usize))
    else {
        return 1;
    };

    // SAFETY: rgba_data must have at least data_size bytes
    let rgba_data_slice = unsafe { std::slice::from_raw_parts_mut(rgba_data, data_size) };
    mirror_horizontal(rgba_data_slice, width, height);
    mirror_vertical(rgba_data_slice, width, height);
    0
}

fn mirror_vertical(buf: &mut [u8], width: u32, height: u32) {
    let stride = width * PIXEL_BYTES;
    assert_eq!(buf.len() as u32, stride * height, "invalid RGBA buffer size");

    for y in 0..height / 2 {
        let top_start = y * stride;
        let bottom_start = (height - 1 - y) * stride;

        for i in 0..stride {
            buf.swap((top_start + i) as usize, (bottom_start + i) as usize);
        }
    }
}

fn mirror_horizontal(buf: &mut [u8], width: u32, height: u32) {
    let stride = width * 4;
    assert_eq!(buf.len() as u32, stride * height, "invalid RGBA buffer size");

    for line in buf.chunks_exact_mut(stride as usize) {
        for x in 0..width / 2 {
            let a = x * 4;
            let b = (width - 1 - x) * 4;
            line.swap(a as usize, b as usize);
            line.swap((a + 1) as usize, (b + 1) as usize);
            line.swap((a + 2) as usize, (b + 2) as usize);
            line.swap((a + 3) as usize, (b + 3) as usize);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}