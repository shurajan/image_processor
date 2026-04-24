use std::fmt;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PluginError {
    Ok = 0,
    InvalidSize = 1,
    UnknownError = 2,
    InvalidParams = 3,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "Ok"),
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
            0 => PluginError::Ok,
            1 => PluginError::InvalidSize,
            3 => PluginError::InvalidParams,
            _ => PluginError::UnknownError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        assert_eq!(PluginError::Ok.to_string(), "Ok");
        assert_eq!(PluginError::InvalidSize.to_string(), "invalid image size");
        assert_eq!(PluginError::UnknownError.to_string(), "unknown error");
        assert_eq!(PluginError::InvalidParams.to_string(), "invalid params");
    }

    #[test]
    fn repr_values() {
        assert_eq!(PluginError::Ok as i32, 0);
        assert_eq!(PluginError::InvalidSize as i32, 1);
        assert_eq!(PluginError::UnknownError as i32, 2);
        assert_eq!(PluginError::InvalidParams as i32, 3);
    }

    #[test]
    fn from_i32_known_codes() {
        assert_eq!(PluginError::from(0), PluginError::Ok);
        assert_eq!(PluginError::from(1), PluginError::InvalidSize);
        assert_eq!(PluginError::from(3), PluginError::InvalidParams);
    }

    #[test]
    fn from_i32_unknown_codes_map_to_unknown_error() {
        assert_eq!(PluginError::from(2), PluginError::UnknownError);
        assert_eq!(PluginError::from(-1), PluginError::UnknownError);
        assert_eq!(PluginError::from(99), PluginError::UnknownError);
    }

    #[test]
    fn copy_clone_and_eq() {
        let e = PluginError::InvalidSize;
        let c = e;
        assert_eq!(e, c);
        assert_eq!(e.clone(), PluginError::InvalidSize);
    }
}
