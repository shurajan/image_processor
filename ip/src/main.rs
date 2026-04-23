mod plugin;

use std::ffi::CString;
use clap::Parser;
use std::path::PathBuf;
use crate::plugin::Plugin;

#[derive(Parser, Debug)]
#[command(
    name = "ip",
    version = "0.1.0",
    about = "Reads an input RGB file and ang converts it with plugin",
    long_about = None,
    arg_required_else_help = true
)]

//run --package ip --bin ip -- -i tests/data/2.png
struct Args {
    /// Input file path
    #[arg(short = 'i', long, value_name = "FILE")]
    input: PathBuf,

    /// Output file path
    #[arg(short = 'o', long, value_name = "FILE")]
    output: PathBuf,

    /// Plugin name
    #[arg(short = 'p', long, value_name = "PLUGIN_NAME")]
    plugin: String,

    /// Path to file with the plugins' params
    #[arg(short = 'd', long, value_name = "FILE")]
    pub params: PathBuf,

    /// Path to plugins directory
    #[arg(short = 'l', long, default_value = "target/debug", value_name = "DIR")]
    pub plugin_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let img = image::open(&args.input)?;
    let rgba = img.to_rgba8();
    let mut rgba_h = rgba.clone();
    let (width, height) = rgba.dimensions();

    let plugin_filename = libloading::library_filename(args.plugin);
    let plugin_file =args.plugin_path.join(plugin_filename);

    let plugin_lib = Plugin::new(plugin_file)?;
    let interface = plugin_lib.interface()?;
    let c_params = CString::new("")?;

    let err =
        unsafe { (interface.process_image)(width, height, rgba_h.as_mut_ptr(), c_params.as_ptr()) };

    // let mut rgba_v = rgba.clone();
    // let mut rgba_a = rgba.clone();
    // let mut rgba_bg = rgba.clone();
    // let mut rgba_bb = rgba.clone();


    // mirror_horizontal(rgba_h.as_mut(), width as usize, height as usize);
    // rgba_h.save("tests/output/test_hor.png")?;
    //
    // mirror_vertical(rgba_v.as_mut(), width as usize, height as usize);
    // rgba_v.save("tests/output/test_ver.png")?;
    //
    // mirror_horizontal(rgba_a.as_mut(), width as usize, height as usize);
    // mirror_vertical(rgba_a.as_mut(), width as usize, height as usize);
    // rgba_a.save("tests/output/test_all.png")?;
    //
    // blur_gauss(rgba_bg.as_mut(), width as usize, height as usize);
    // rgba_bg.save("tests/output/test_blur_gauss.png")?;
    //
    // blur_box(rgba_bb.as_mut(), width as usize, height as usize);
    rgba_h.save("tests/output/test_invert.png")?;

    Ok(())
}


fn mirror_vertical(buf: &mut [u8], width: usize, height: usize) {
    let stride = width * 4;
    assert_eq!(buf.len(), stride * height, "invalid RGBA buffer size");

    for y in 0..height / 2 {
        let top_start = y * stride;
        let bottom_start = (height - 1 - y) * stride;

        for i in 0..stride {
            buf.swap(top_start + i, bottom_start + i);
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

const BLUR_RADIUS: i32 = 9;
const BLUR_SIGMA: f32 = 3.0;
const BOX_BLUR_RADIUS: i32 = 9;
const BOX_BLUR_ITERATIONS: usize = 3;

fn blur_gauss(buf: &mut [u8], width: usize, height: usize) {
    assert_eq!(buf.len(), width * 4 * height, "invalid RGBA buffer size");

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
