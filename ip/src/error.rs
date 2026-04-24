use plugin_error::PluginError;
use std::path::PathBuf;
use thiserror::Error;

/// All errors that can occur during image processing in the `ip` binary.
#[derive(Debug, Error)]
pub enum AppError {
    /// App failed to parse the command line with `clap`.
    #[error(transparent)]
    ArgumentParsing(#[from] clap::Error),
    /// The specified input image file does not exist.
    #[error("input file not found: {0}")]
    InputFileNotFound(PathBuf),
    /// The JSON params file does not exist.
    #[error("params file not found: {0}")]
    ParamsFileNotFound(PathBuf),
    /// The plugin directory does not exist.
    #[error("plugin directory not found: {0}")]
    PluginDirNotFound(PathBuf),
    /// The parent directory of the output path does not exist.
    #[error("output directory not found: {0}")]
    OutputDirNotFound(PathBuf),
    /// The params string contains an interior NUL byte and cannot be passed to the plugin.
    #[error("plugin params contain an interior NUL byte: {0}")]
    InvalidPluginParams(#[from] std::ffi::NulError),
    /// A standard I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An error from the `image` crate while opening or saving an image.
    #[error(transparent)]
    Image(#[from] image::ImageError),
    /// A dynamic-library loading error from `libloading`.
    #[error(transparent)]
    Plugin(#[from] libloading::Error),
    /// The plugin's `process_image` returned a non-zero error code.
    #[error("plugin processing failed: {0}")]
    PluginProcessingFailed(#[from] PluginError),
}
