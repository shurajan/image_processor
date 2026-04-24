use plugin_error::PluginError;
use std::ffi::{c_char, c_uchar};

// void process_image(
//      uint32_t width,
//      uint32_t height,
//      uint8_t* rgba_data,
//      const char* params
// );

#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut c_uchar,
    _params: *const c_char,
) -> i32 {
    let Some(data_size) = (width as usize)
        .checked_mul(height as usize)
        .and_then(|res| res.checked_mul(PIXEL_BYTES as usize))
    else {
        return PluginError::InvalidSize as i32;
    };

    // SAFETY: rgba_data must have at least data_size bytes
    let rgba_data_slice = unsafe { std::slice::from_raw_parts_mut(rgba_data, data_size) };
    blur_box(rgba_data_slice, width as usize, height as usize);
    0
}

const PIXEL_BYTES: usize = 4;
#[allow(dead_code)]
const BLUR_RADIUS: i32 = 9;
#[allow(dead_code)]
const BLUR_SIGMA: f32 = 3.0;
const BOX_BLUR_RADIUS: i32 = 9;
const BOX_BLUR_ITERATIONS: usize = 3;

#[allow(dead_code)]
fn blur_gauss(buf: &mut [u8], width: usize, height: usize) {
    assert_eq!(
        buf.len(),
        width * PIXEL_BYTES * height,
        "invalid RGBA buffer size"
    );

    let kernel_size = (2 * BLUR_RADIUS + 1) as usize;
    let mut kernel = vec![0f32; kernel_size];
    let mut sum = 0f32;
    for i in 0..kernel_size {
        let x = i as i32 - BLUR_RADIUS;
        let v = (-(x * x) as f32 / (2.0 * BLUR_SIGMA * BLUR_SIGMA)).exp();
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
                let sx = (x as i32 + ki as i32 - BLUR_RADIUS).clamp(0, width as i32 - 1) as usize;
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
                let sy = (y as i32 + ki as i32 - BLUR_RADIUS).clamp(0, height as i32 - 1) as usize;
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

fn blur_box(buf: &mut [u8], width: usize, height: usize) {
    assert_eq!(buf.len(), width * 4 * height, "invalid RGBA buffer size");

    let kernel_size = (2 * BOX_BLUR_RADIUS + 1) as usize;
    let w = 1.0f32 / kernel_size as f32;

    let mut temp = vec![0u8; buf.len()];

    for _ in 0..BOX_BLUR_ITERATIONS {
        // horizontal
        for y in 0..height {
            for x in 0..width {
                let (mut r, mut g, mut b, mut a) = (0f32, 0f32, 0f32, 0f32);
                for ki in 0..kernel_size {
                    let sx = (x as i32 + ki as i32 - BOX_BLUR_RADIUS).clamp(0, width as i32 - 1)
                        as usize;
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
                    let sy = (y as i32 + ki as i32 - BOX_BLUR_RADIUS).clamp(0, height as i32 - 1)
                        as usize;
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
    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
