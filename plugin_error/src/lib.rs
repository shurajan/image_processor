use std::fmt;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PluginError {
    InvalidSize = 1,
    UnknownError = 2,
    InvalidParams = 3,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize => write!(f, "invalid image size"),
            Self::UnknownError => write!(f, "unknown error"),
            Self::InvalidParams => write!(f, "invalid params"),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<i32> for PluginError {
    fn from(code: i32) -> Self {
        match code {
            1 => PluginError::InvalidSize,
            3 => PluginError::InvalidParams,
            _ => PluginError::UnknownError,
        }
    }
}
