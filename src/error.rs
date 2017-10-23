//! Engine error types.

use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;
use std::result::Result as StdResult;

use config::ConfigError;
use ecs::error::BoxedErr;
use renderer;

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
    System(BoxedErr),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Application => "Application error!",
            Error::Config(_) => "Configuration error!",
            Error::System(_) => "System error!",
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
            Error::System(ref e) => write!(fmt, "System creation failed: {}", e),
        }
    }
}

impl From<BoxedErr> for Error {
    fn from(e: BoxedErr) -> Self {
        Error::System(e)
    }
}

impl From<renderer::error::Error> for Error {
    fn from(err: renderer::error::Error) -> Self {
        Error::System(BoxedErr::new(err))
    }
}
