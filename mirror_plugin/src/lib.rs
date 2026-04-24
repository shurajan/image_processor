use plugin_error::PluginError;
use serde::Deserialize;
use std::ffi::{CStr, c_char, c_uchar};
use std::panic::{AssertUnwindSafe, catch_unwind};

const PIXEL_BYTES: u32 = 4;

#[derive(Deserialize)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool,
}

/// Mirrors an RGBA8 image buffer in-place along one or both axes.
///
/// `params` must be a null-terminated JSON string:
/// `{"horizontal":<bool>,"vertical":<bool>}`
///
/// Returns a [`PluginError`] code cast to `i32` (`0` on success).
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

    let result = catch_unwind(AssertUnwindSafe(move || {
        if mirror_params.horizontal {
            mirror_horizontal(rgba_data_slice, width as usize, height as usize);
        }
        if mirror_params.vertical {
            mirror_vertical(rgba_data_slice, width, height);
        }
        PluginError::Ok as i32
    }));

    result.unwrap_or(PluginError::UnknownError as i32)
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
    use plugin_error::PluginError;
    use std::ffi::CString;

    fn pixel(r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
        [r, g, b, a]
    }

    fn make_buf(pixels: &[[u8; 4]]) -> Vec<u8> {
        pixels.iter().flat_map(|p| p.iter().copied()).collect()
    }

    // 2×1 image [RED | BLUE] → after horizontal mirror → [BLUE | RED]
    #[test]
    fn mirror_horizontal_swaps_pixels_in_row() {
        let red = pixel(255, 0, 0, 255);
        let blue = pixel(0, 0, 255, 255);
        let mut buf = make_buf(&[red, blue]);
        mirror_horizontal(&mut buf, 2, 1);
        assert_eq!(&buf[0..4], &blue, "first pixel should be blue");
        assert_eq!(&buf[4..8], &red, "second pixel should be red");
    }

    // 1×2 image [RED / BLUE] → after vertical mirror → [BLUE / RED]
    #[test]
    fn mirror_vertical_swaps_rows() {
        let red = pixel(255, 0, 0, 255);
        let blue = pixel(0, 0, 255, 255);
        let mut buf = make_buf(&[red, blue]);
        mirror_vertical(&mut buf, 1, 2);
        assert_eq!(&buf[0..4], &blue, "top pixel should be blue");
        assert_eq!(&buf[4..8], &red, "bottom pixel should be red");
    }

    // 2×2: [TL TR / BL BR] → h-mirror → [TR TL / BR BL] → v-mirror → [BR BL / TR TL]
    #[test]
    fn mirror_both_axes_2x2() {
        let tl = pixel(1, 0, 0, 255);
        let tr = pixel(2, 0, 0, 255);
        let bl = pixel(3, 0, 0, 255);
        let br = pixel(4, 0, 0, 255);
        let mut buf = make_buf(&[tl, tr, bl, br]);
        mirror_horizontal(&mut buf, 2, 2);
        mirror_vertical(&mut buf, 2, 2);
        assert_eq!(&buf[0..4], &br, "top-left should be BR");
        assert_eq!(&buf[4..8], &bl, "top-right should be BL");
        assert_eq!(&buf[8..12], &tr, "bottom-left should be TR");
        assert_eq!(&buf[12..16], &tl, "bottom-right should be TL");
    }

    #[test]
    fn mirror_horizontal_twice_is_identity() {
        let orig = make_buf(&[pixel(1, 2, 3, 4), pixel(5, 6, 7, 8), pixel(9, 10, 11, 12)]);
        let mut buf = orig.clone();
        mirror_horizontal(&mut buf, 3, 1);
        mirror_horizontal(&mut buf, 3, 1);
        assert_eq!(buf, orig);
    }

    #[test]
    fn mirror_vertical_twice_is_identity() {
        let orig = make_buf(&[pixel(1, 2, 3, 4), pixel(5, 6, 7, 8)]);
        let mut buf = orig.clone();
        mirror_vertical(&mut buf, 1, 2);
        mirror_vertical(&mut buf, 1, 2);
        assert_eq!(buf, orig);
    }

    #[test]
    fn mirror_horizontal_uniform_unchanged() {
        let mut buf = vec![128u8; 4 * 4 * 4];
        let original = buf.clone();
        mirror_horizontal(&mut buf, 4, 4);
        assert_eq!(buf, original);
    }

    #[test]
    fn process_image_null_params_returns_invalid_params() {
        let (w, h) = (2u32, 2u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), std::ptr::null()) };
        assert_eq!(ret, PluginError::InvalidParams as i32);
    }

    #[test]
    fn process_image_invalid_json_returns_invalid_params() {
        let (w, h) = (2u32, 2u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let params = CString::new("not valid json").unwrap();
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), params.as_ptr()) };
        assert_eq!(ret, PluginError::InvalidParams as i32);
    }

    #[test]
    fn process_image_horizontal_mirror_returns_ok() {
        let (w, h) = (2u32, 1u32);
        let mut buf = make_buf(&[pixel(255, 0, 0, 255), pixel(0, 0, 255, 255)]);
        let params = CString::new(r#"{"horizontal":true,"vertical":false}"#).unwrap();
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), params.as_ptr()) };
        assert_eq!(ret, PluginError::Ok as i32);
        // pixels should be swapped
        assert_eq!(&buf[0..4], &pixel(0, 0, 255, 255));
        assert_eq!(&buf[4..8], &pixel(255, 0, 0, 255));
    }

    #[test]
    fn process_image_vertical_mirror_returns_ok() {
        let (w, h) = (1u32, 2u32);
        let mut buf = make_buf(&[pixel(255, 0, 0, 255), pixel(0, 0, 255, 255)]);
        let params = CString::new(r#"{"horizontal":false,"vertical":true}"#).unwrap();
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), params.as_ptr()) };
        assert_eq!(ret, PluginError::Ok as i32);
        assert_eq!(&buf[0..4], &pixel(0, 0, 255, 255));
        assert_eq!(&buf[4..8], &pixel(255, 0, 0, 255));
    }

    #[test]
    fn process_image_no_mirror_returns_ok_and_unchanged() {
        let (w, h) = (2u32, 2u32);
        let mut buf = vec![128u8; (w * h * 4) as usize];
        let original = buf.clone();
        let params = CString::new(r#"{"horizontal":false,"vertical":false}"#).unwrap();
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), params.as_ptr()) };
        assert_eq!(ret, PluginError::Ok as i32);
        assert_eq!(buf, original);
    }
}
