use std::fmt;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PluginError {
    InvalidSize = 1,
    UnknownError = 2,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize => write!(f, "invalid image size"),
            Self::UnknownError => write!(f, "unknown error"),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<i32> for PluginError {
    fn from(code: i32) -> Self {
        match code {
            1 => PluginError::InvalidSize,
            _ => PluginError::UnknownError,
        }
    }
}
