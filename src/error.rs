//! Engine error types.

use config::ConfigError;
use core;
use renderer;
use std::error::Error as StdError;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
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
    /// Core error.
    Core(core::Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Application => "Application error!",
            Error::Config(_) => "Configuration error!",
            Error::Core(_) => "Core error!",
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
            Error::Core(ref e) => write!(fmt, "System creation failed: {}", e),
        }
    }
}

impl From<core::Error> for Error {
    fn from(e: core::Error) -> Self {
        Error::Core(e)
    }
}

impl From<renderer::error::Error> for Error {
    fn from(err: renderer::error::Error) -> Self {
        Error::Core(core::Error::with_chain(err, "Renderer error"))
    }
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Error::Config(err)
    }
}
