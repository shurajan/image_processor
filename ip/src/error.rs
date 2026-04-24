use plugin_error::PluginError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    ArgumentParsing(#[from] clap::Error),
    #[error("input file not found: {0}")]
    InputFileNotFound(PathBuf),
    #[error("params file not found: {0}")]
    ParamsFileNotFound(PathBuf),
    #[error("plugin directory not found: {0}")]
    PluginDirNotFound(PathBuf),
    #[error("output directory not found: {0}")]
    OutputDirNotFound(PathBuf),
    #[error("plugin params contain an interior NUL byte: {0}")]
    InvalidPluginParams(#[from] std::ffi::NulError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    Plugin(#[from] libloading::Error),
    #[error("plugin processing failed: {0}")]
    PluginProcessingFailed(#[from] PluginError),
}
