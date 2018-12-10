//! Engine error types.

use std::{
    error::Error as StdError,
    fmt::Result as FmtResult,
    fmt::{Display, Formatter},
    io,
    result::Result as StdResult,
};

use crate::{config::ConfigError, core, renderer, state::StateError};

/// Engine result type.
pub type Result<T> = StdResult<T, Error>;

/// Common error type.
#[derive(Debug)]
pub enum Error {
    /// Application error.
    Application,
    /// StateMachine error
    StateMachine(StateError),
    /// Asset management error.
    // Asset(AssetError),
    /// Configuration error.
    Config(ConfigError),
    /// Core error.
    Core(core::Error),
    /// I/O Error
    Io(io::Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Application => "Application error!",
            Error::Config(_) => "Configuration error!",
            Error::Core(_) => "Core error!",
            Error::StateMachine(_) => "StateMachine error!",
            Error::Io(_) => "I/O error!",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Error::Config(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Error::Application => write!(fmt, "Application initialization failed!"),
            Error::Config(ref e) => write!(fmt, "Configuration loading failed: {}", e),
            Error::Core(ref e) => write!(fmt, "System creation failed: {}", e),
            Error::StateMachine(ref e) => write!(fmt, "Error in state machine: {}", e),
            Error::Io(ref e) => write!(fmt, "I/O Error: {}", e),
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

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
