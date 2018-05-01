//! Engine error types.

use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;

use config::ConfigError;
use core::Error as CoreError;
use failure::{Backtrace, Context, Fail};
use renderer::error::Error as RendererError;

/// Engine result type.
pub type Result<T> = StdResult<T, Error>;

/// Common error type.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// The different types of amethyst errors
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Fail)]
pub enum ErrorKind {
    /// Animation subsystem error.
    #[fail(display = "An error occured in the animation subsystem.")]
    Animation,
    /// Asset management subsystem error.
    #[fail(display = "An error occured in the assets subsystem.")]
    Assets,
    /// Audio subsystem error.
    #[fail(display = "An error occured in the audio subsystem.")]
    Audio,
    /// Configuration subsystem error.
    #[fail(display = "An error occured in the config subsystem.")]
    Config,
    /// Controls subsystem error.
    #[fail(display = "An error occured in the controls subsystem.")]
    Controls,
    /// Core subsystem error.
    #[fail(display = "An error occured in the core subsystem.")]
    Core,
    /// General error.
    #[fail(display = "A general amethyst error occured.")]
    General,
    /// Gltf subsystem error.
    #[fail(display = "An error occured in the gltf subsystem.")]
    Gltf,
    /// Input subsystem error.
    #[fail(display = "An error occured in the input subsystem.")]
    Input,
    /// Renderer subsystem error.
    #[fail(display = "An error occured in the renderer subsystem.")]
    Renderer,
    /// Ui subsystem error.
    #[fail(display = "An error occured in the UI subsystem.")]
    Ui,
    /// Utils subsystem error.
    #[fail(display = "An error occured in the utils subsystem.")]
    Utils,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Error { inner }
    }
}

impl From<ConfigError> for Error {
    fn from(e: ConfigError) -> Self {
        e.context(ErrorKind::Config).into()
    }
}

impl From<CoreError> for Error {
    fn from(e: CoreError) -> Self {
        e.context(ErrorKind::Core).into()
    }
}

impl From<RendererError> for Error {
    fn from(e: RendererError) -> Self {
        e.context(ErrorKind::Renderer).into()
    }
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Error::Config(err)
    }
}

