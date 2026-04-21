use anyhow::Context;
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use image::{GenericImageView, RgbaImage};

#[derive(Parser, Debug)]
#[command(
    name = "ip",
    version = "0.1.0",
    about = "Reads an input RGB file and ang converts it with plugin",
    long_about = None,
    arg_required_else_help = true
)]
struct Args {
    /// Input file path
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let img = image::open(&args.input)?;
    let mut rgba = img.to_rgba8();

    let mut rgba_h = rgba.clone();
    let mut rgba_v = rgba.clone();
    let mut rgba_a = rgba.clone();

    let (width, height) = rgba.dimensions();
    mirror_horizontal(rgba_h.as_mut(), width as usize, height as usize);
    rgba_h.save("hor.png")?;

    mirror_vertical(rgba_v.as_mut(), width as usize, height as usize);
    rgba_v.save("ver.png")?;

    mirror_horizontal(rgba_a.as_mut(), width as usize, height as usize);
    mirror_vertical(rgba_a.as_mut(), width as usize, height as usize);
    rgba_a.save("all.png")?;

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


fn mirror_both_image(rgba: &RgbaImage) -> RgbaImage {
    let row = rgba.to_vec();

    let data = vec![1,2,3,4,5,6,7,8];

    let mut parts: Vec<&[u8]> = data.chunks(4).collect();

    parts.reverse();

    println!("{:?}", parts);

    let pixels = rgba.pixels();

    // Rgba([83, 174, 246, 205])
    // Rgba([82, 175, 245, 205])


    // for pixel in pixels {
    //     println!("_{} {} {} {}_", pixel[0], pixel[1], pixel[2], pixel[3]);
    // }
    println!("end");

    rgba.clone()
}