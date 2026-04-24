use plugin_error::PluginError;
use serde::Deserialize;
use std::ffi::{CStr, c_char, c_uchar};

const PIXEL_BYTES: u32 = 4;

#[derive(Deserialize)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool,
}

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
        return PluginError::InvalidSize as i32;
    };

    if params.is_null() {
        return PluginError::InvalidParams as i32;
    }

    // SAFETY: caller must provide a valid null-terminated UTF-8 string
    let params_str = unsafe { CStr::from_ptr(params) };
    let Ok(params_str) = params_str.to_str() else {
        return PluginError::InvalidParams as i32;
    };
    let Ok(mirror_params) = serde_json::from_str::<MirrorParams>(params_str) else {
        return PluginError::InvalidParams as i32;
    };

    // SAFETY: rgba_data must have at least data_size bytes
    let rgba_data_slice = unsafe { std::slice::from_raw_parts_mut(rgba_data, data_size) };

    if mirror_params.horizontal {
        mirror_horizontal(rgba_data_slice, width as usize, height as usize);
    }
    if mirror_params.vertical {
        mirror_vertical(rgba_data_slice, width, height);
    }

    PluginError::Ok as i32
}

fn mirror_vertical(buf: &mut [u8], width: u32, height: u32) {
    let stride = width * PIXEL_BYTES;
    assert_eq!(
        buf.len() as u32,
        stride * height,
        "invalid RGBA buffer size"
    );

    for y in 0..height / 2 {
        let top_start = y * stride;
        let bottom_start = (height - 1 - y) * stride;

        for i in 0..stride {
            buf.swap((top_start + i) as usize, (bottom_start + i) as usize);
        }
    }
}

fn mirror_horizontal(buf: &mut [u8], width: usize, height: usize) {
    let stride = width * 4;
    assert_eq!(buf.len(), stride * height, "invalid RGBA buffer size");

    for line in buf.chunks_exact_mut(stride) {
        for x in 0..width / 2 {
            let a = x * 4;
            let b = (width - 1 - x) * 4;
            line.swap(a, b);
            line.swap(a + 1, b + 1);
            line.swap(a + 2, b + 2);
            line.swap(a + 3, b + 3);
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
