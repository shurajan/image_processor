use plugin_error::PluginError;
use serde::Deserialize;
use std::ffi::{CStr, c_char, c_uchar};

const PIXEL_BYTES: usize = 4;

#[derive(Deserialize)]
#[serde(tag = "method", rename_all = "lowercase")]
enum BlurParams {
    Box { radius: i32, iterations: usize },
    Gauss { radius: i32, sigma: f32 },
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

    match blur_params {
        BlurParams::Box { radius, iterations } => {
            blur_box(rgba_data_slice, width as usize, height as usize, radius, iterations);
        }
        BlurParams::Gauss { radius, sigma } => {
            blur_gauss(rgba_data_slice, width as usize, height as usize, radius, sigma);
        }
    }

    PluginError::Ok as i32
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
                    let sy =
                        (y as i32 + ki as i32 - radius).clamp(0, height as i32 - 1) as usize;
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

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
