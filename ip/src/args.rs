use crate::error::AppError;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ip",
    version = "0.1.0",
    about = "Reads an input RGB file and converts it with plugin",
    long_about = None,
    arg_required_else_help = true
)]

/// Validated command-line arguments for the `ip` tool.
pub struct Args {
    /// Input file path
    #[arg(short = 'i', long, value_name = "FILE")]
    pub input: PathBuf,

    /// Output file path
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: PathBuf,

    /// Plugin name
    #[arg(short = 'p', long, value_name = "PLUGIN_NAME")]
    pub plugin: String,

    /// Path to file with the plugins' params
    #[arg(short = 'd', long, value_name = "FILE")]
    pub params: PathBuf,

    /// Path to plugins directory
    #[arg(short = 'l', long, default_value = "target/debug", value_name = "DIR")]
    pub plugin_path: PathBuf,
}

/// Parses and validates command-line arguments, returning an error if any required paths are invalid.
pub(crate) fn parse_args() -> Result<Args, AppError> {
    let args = Args::try_parse().map_err(AppError::ArgumentParsing)?;

    if !args.input.is_file() {
        return Err(AppError::InputFileNotFound(args.input.clone()));
    }

    if !args.params.is_file() {
        return Err(AppError::ParamsFileNotFound(args.params.clone()));
    }

    if !args.plugin_path.is_dir() {
        return Err(AppError::PluginDirNotFound(args.plugin_path.clone()));
    }

    if let Some(parent) = args.output.parent()
        && !parent.as_os_str().is_empty()
        && !parent.is_dir()
    {
        return Err(AppError::OutputDirNotFound(parent.to_path_buf()));
    }

    Ok(args)
}
