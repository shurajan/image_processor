use plugin_error::PluginError;
use serde::Deserialize;
use std::ffi::{CStr, c_char, c_uchar};
use std::panic::{AssertUnwindSafe, catch_unwind};

const PIXEL_BYTES: usize = 4;

#[derive(Deserialize)]
#[serde(tag = "method", rename_all = "lowercase")]
enum BlurParams {
    Box { radius: i32, iterations: usize },
    Gauss { radius: i32, sigma: f32 },
}

/// Applies a blur effect to an RGBA8 image buffer in-place.
///
/// `params` must be a null-terminated JSON string in one of these forms:
/// - `{"method":"box","radius":<i32>,"iterations":<usize>}`
/// - `{"method":"gauss","radius":<i32>,"sigma":<f32>}`
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
        .and_then(|res| res.checked_mul(PIXEL_BYTES))
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
    let Ok(blur_params) = serde_json::from_str::<BlurParams>(params_str) else {
        return PluginError::InvalidParams as i32;
    };

    // SAFETY: rgba_data must have at least data_size bytes
    let rgba_data_slice = unsafe { std::slice::from_raw_parts_mut(rgba_data, data_size) };

    let result = catch_unwind(AssertUnwindSafe(move || {
        match blur_params {
            BlurParams::Box { radius, iterations } => {
                blur_box(
                    rgba_data_slice,
                    width as usize,
                    height as usize,
                    radius,
                    iterations,
                );
            }
            BlurParams::Gauss { radius, sigma } => {
                blur_gauss(
                    rgba_data_slice,
                    width as usize,
                    height as usize,
                    radius,
                    sigma,
                );
            }
        }
        PluginError::Ok as i32
    }));

    result.unwrap_or(PluginError::UnknownError as i32)
}

fn blur_gauss(buf: &mut [u8], width: usize, height: usize, radius: i32, sigma: f32) {
    assert_eq!(
        buf.len(),
        width * PIXEL_BYTES * height,
        "invalid RGBA buffer size"
    );

    let kernel_size = (2 * radius + 1) as usize;
    let mut kernel = vec![0f32; kernel_size];
    let mut sum = 0f32;
    for i in 0..kernel_size {
        let x = i as i32 - radius;
        let v = (-(x * x) as f32 / (2.0 * sigma * sigma)).exp();
        kernel[i] = v;
        sum += v;
    }
    for k in &mut kernel {
        *k /= sum;
    }

    let mut temp = vec![0u8; buf.len()];

    // horizontal
    for y in 0..height {
        for x in 0..width {
            let (mut r, mut g, mut b, mut a) = (0f32, 0f32, 0f32, 0f32);
            for (ki, &w) in kernel.iter().enumerate() {
                let sx = (x as i32 + ki as i32 - radius).clamp(0, width as i32 - 1) as usize;
                let idx = (y * width + sx) * 4;
                r += buf[idx] as f32 * w;
                g += buf[idx + 1] as f32 * w;
                b += buf[idx + 2] as f32 * w;
                a += buf[idx + 3] as f32 * w;
            }
            let out = (y * width + x) * 4;
            temp[out] = r.round() as u8;
            temp[out + 1] = g.round() as u8;
            temp[out + 2] = b.round() as u8;
            temp[out + 3] = a.round() as u8;
        }
    }

    // vertical
    for y in 0..height {
        for x in 0..width {
            let (mut r, mut g, mut b, mut a) = (0f32, 0f32, 0f32, 0f32);
            for (ki, &w) in kernel.iter().enumerate() {
                let sy = (y as i32 + ki as i32 - radius).clamp(0, height as i32 - 1) as usize;
                let idx = (sy * width + x) * 4;
                r += temp[idx] as f32 * w;
                g += temp[idx + 1] as f32 * w;
                b += temp[idx + 2] as f32 * w;
                a += temp[idx + 3] as f32 * w;
            }
            let out = (y * width + x) * 4;
            buf[out] = r.round() as u8;
            buf[out + 1] = g.round() as u8;
            buf[out + 2] = b.round() as u8;
            buf[out + 3] = a.round() as u8;
        }
    }
}

fn blur_box(buf: &mut [u8], width: usize, height: usize, radius: i32, iterations: usize) {
    assert_eq!(buf.len(), width * 4 * height, "invalid RGBA buffer size");

    let kernel_size = (2 * radius + 1) as usize;
    let w = 1.0f32 / kernel_size as f32;

    let mut temp = vec![0u8; buf.len()];

    for _ in 0..iterations {
        // horizontal
        for y in 0..height {
            for x in 0..width {
                let (mut r, mut g, mut b, mut a) = (0f32, 0f32, 0f32, 0f32);
                for ki in 0..kernel_size {
                    let sx = (x as i32 + ki as i32 - radius).clamp(0, width as i32 - 1) as usize;
                    let idx = (y * width + sx) * 4;
                    r += buf[idx] as f32;
                    g += buf[idx + 1] as f32;
                    b += buf[idx + 2] as f32;
                    a += buf[idx + 3] as f32;
                }
                let out = (y * width + x) * 4;
                temp[out] = (r * w).round() as u8;
                temp[out + 1] = (g * w).round() as u8;
                temp[out + 2] = (b * w).round() as u8;
                temp[out + 3] = (a * w).round() as u8;
            }
        }

        // vertical pass
        for y in 0..height {
            for x in 0..width {
                let (mut r, mut g, mut b, mut a) = (0f32, 0f32, 0f32, 0f32);
                for ki in 0..kernel_size {
                    let sy = (y as i32 + ki as i32 - radius).clamp(0, height as i32 - 1) as usize;
                    let idx = (sy * width + x) * 4;
                    r += temp[idx] as f32;
                    g += temp[idx + 1] as f32;
                    b += temp[idx + 2] as f32;
                    a += temp[idx + 3] as f32;
                }
                let out = (y * width + x) * 4;
                buf[out] = (r * w).round() as u8;
                buf[out + 1] = (g * w).round() as u8;
                buf[out + 2] = (b * w).round() as u8;
                buf[out + 3] = (a * w).round() as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_error::PluginError;
    use std::ffi::CString;

    fn uniform_rgba(width: usize, height: usize, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
        let mut buf = vec![0u8; width * height * 4];
        for i in 0..width * height {
            buf[i * 4] = r;
            buf[i * 4 + 1] = g;
            buf[i * 4 + 2] = b;
            buf[i * 4 + 3] = a;
        }
        buf
    }

    #[test]
    fn blur_box_uniform_image_stays_uniform() {
        let (w, h) = (8, 8);
        let mut buf = uniform_rgba(w, h, 100, 150, 200, 255);
        blur_box(&mut buf, w, h, 2, 3);
        for i in 0..w * h {
            assert_eq!(buf[i * 4], 100, "R changed at pixel {i}");
            assert_eq!(buf[i * 4 + 1], 150, "G changed at pixel {i}");
            assert_eq!(buf[i * 4 + 2], 200, "B changed at pixel {i}");
            assert_eq!(buf[i * 4 + 3], 255, "A changed at pixel {i}");
        }
    }

    #[test]
    fn blur_box_zero_radius_is_identity() {
        let (w, h) = (4, 4);
        let mut buf = uniform_rgba(w, h, 10, 20, 30, 40);
        buf[0] = 255;
        let original = buf.clone();
        blur_box(&mut buf, w, h, 0, 1);
        assert_eq!(buf, original);
    }

    #[test]
    fn blur_gauss_uniform_image_stays_uniform() {
        let (w, h) = (8, 8);
        let mut buf = uniform_rgba(w, h, 128, 64, 32, 255);
        blur_gauss(&mut buf, w, h, 3, 1.5);
        for i in 0..w * h {
            // allow ±1 for floating-point rounding
            assert!((buf[i * 4] as i32 - 128).abs() <= 1, "R at {i}");
            assert!((buf[i * 4 + 1] as i32 - 64).abs() <= 1, "G at {i}");
            assert!((buf[i * 4 + 2] as i32 - 32).abs() <= 1, "B at {i}");
            assert_eq!(buf[i * 4 + 3], 255, "A changed at pixel {i}");
        }
    }

    #[test]
    #[should_panic(expected = "invalid RGBA buffer size")]
    fn blur_box_wrong_buffer_size_panics() {
        let mut buf = vec![0u8; 10];
        blur_box(&mut buf, 4, 4, 1, 1);
    }

    #[test]
    #[should_panic(expected = "invalid RGBA buffer size")]
    fn blur_gauss_wrong_buffer_size_panics() {
        let mut buf = vec![0u8; 10];
        blur_gauss(&mut buf, 4, 4, 1, 1.0);
    }

    #[test]
    fn process_image_box_params_returns_ok() {
        let (w, h) = (4u32, 4u32);
        let mut buf = uniform_rgba(w as usize, h as usize, 128, 128, 128, 255);
        let params = CString::new(r#"{"method":"box","radius":1,"iterations":2}"#).unwrap();
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), params.as_ptr()) };
        assert_eq!(ret, PluginError::Ok as i32);
    }

    #[test]
    fn process_image_gauss_params_returns_ok() {
        let (w, h) = (4u32, 4u32);
        let mut buf = uniform_rgba(w as usize, h as usize, 128, 128, 128, 255);
        let params = CString::new(r#"{"method":"gauss","radius":2,"sigma":1.0}"#).unwrap();
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), params.as_ptr()) };
        assert_eq!(ret, PluginError::Ok as i32);
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
    fn process_image_unknown_method_returns_invalid_params() {
        let (w, h) = (2u32, 2u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let params = CString::new(r#"{"method":"unknown","radius":1}"#).unwrap();
        let ret = unsafe { process_image(w, h, buf.as_mut_ptr(), params.as_ptr()) };
        assert_eq!(ret, PluginError::InvalidParams as i32);
    }
}
