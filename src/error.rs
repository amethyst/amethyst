//! Engine error types.

use assets::AssetError;
use config::ConfigError;

use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;
use std::result::Result as StdResult;

/// Engine result type.
pub type Result<T> = StdResult<T, Error>;

/// Common error type.
#[derive(Debug)]
pub enum Error {
    /// Application error.
    Application,
    /// Asset management error.
    // Asset(AssetError),
    /// Configuration error.
    Config(ConfigError),
    /// System error.
    System,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Application => "Application error!",
            Error::Config(_) => "Configuration error!",
            Error::System => "System error!",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Config(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Error::Application => write!(fmt, "Application initialization failed!"),
            Error::Config(ref e) => write!(fmt, "Configuration loading failed: {}", e),
            Error::System => write!(fmt, "System creation failed!"),
        }
    }
}
