//! Renderer error types.

use gfx;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;
use std::result::Result as StdResult;

/// Renderer result type.
pub type Result<T> = StdResult<T, Error>;

/// Common renderer error type.
#[derive(Debug)]
pub enum Error {
    /// A render target with the given name does not exist.
    NoSuchTarget(String),
    /// Failed to initialize a render pass.
    PassInit(gfx::PipelineStateError<String>),
    /// Failed to create a render target.
    TargetCreation(gfx::CombinedError),
    /// The window handle associated with the renderer has been destroyed.
    WindowDestroyed,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NoSuchTarget(..) => "Target with this name does not exist!",
            Error::PassInit(..) => "Failed to initialize render pass!",
            Error::TargetCreation(..) => "Failed to create target!",
            Error::WindowDestroyed => "Window has been destroyed!",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::PassInit(ref e) => Some(e),
            Error::TargetCreation(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Error::NoSuchTarget(ref e) => write!(fmt, "Nonexistent target: {}", e),
            Error::PassInit(ref e) => write!(fmt, "Pass initialization failed: {}", e),
            Error::TargetCreation(ref e) => write!(fmt, "Target creation failed: {}", e),
            Error::WindowDestroyed => write!(fmt, "Window has been destroyed"),
        }
    }
}

impl From<gfx::CombinedError> for Error {
    fn from(e: gfx::CombinedError) -> Error {
        Error::TargetCreation(e)
    }
}

impl From<gfx::PipelineStateError<String>> for Error {
    fn from(e: gfx::PipelineStateError<String>) -> Error {
        Error::PassInit(e)
    }
}
