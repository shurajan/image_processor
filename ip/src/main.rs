mod args;
mod error;
mod plugin;

use crate::args::parse_args;
use crate::error::AppError;
use crate::plugin::Plugin;
use plugin_error::PluginError;
use std::ffi::CString;
use std::fs;
use std::process::ExitCode;

/// Entry point: runs the tool and maps errors to appropriate exit codes.
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(AppError::ArgumentParsing(err)) => {
            let _ = err.print();
            ExitCode::from(err.exit_code() as u8)
        }
        Err(err) => {
            eprintln!("error: {err}");
            eprintln!("Try 'ip --help' for more information.");
            ExitCode::FAILURE
        }
    }
}

/// Parses arguments, loads the plugin, processes the image in-place, and saves the result.
fn run() -> Result<(), AppError> {
    let args = parse_args()?;

    let img = image::open(&args.input)?;
    let mut rgba_h = img.to_rgba8();
    let (width, height) = rgba_h.dimensions();

    let plugin_filename = libloading::library_filename(args.plugin);
    let plugin_file = args.plugin_path.join(plugin_filename);

    let plugin_lib = Plugin::new(plugin_file)?;
    let interface = plugin_lib.interface()?;
    let params = fs::read_to_string(&args.params)?;
    let c_params = CString::new(params)?;

    let err = (interface.process_image)(width, height, rgba_h.as_mut_ptr(), c_params.as_ptr());
    if err != 0 {
        return Err(PluginError::from(err).into());
    }

    rgba_h.save(&args.output)?;

    Ok(())
}
