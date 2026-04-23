mod plugin;

use crate::plugin::Plugin;
use clap::Parser;
use std::ffi::CString;
use std::path::PathBuf;

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
    let plugin_file = args.plugin_path.join(plugin_filename);

    let plugin_lib = Plugin::new(plugin_file)?;
    let interface = plugin_lib.interface()?;
    let c_params = CString::new("")?;

    let err =
        unsafe { (interface.process_image)(width, height, rgba_h.as_mut_ptr(), c_params.as_ptr()) };

    rgba_h.save("tests/output/test_output.png")?;

    Ok(())
}
