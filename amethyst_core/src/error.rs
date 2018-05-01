//! The error type and associated helpers
use std::fmt;
use std::result::Result as StdResult;

use failure::{Backtrace, Context, Fail};

/// Engine result type.
pub type Result<T> = StdResult<T, Error>;

/// An error in the amethyst core subsystem
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// The types of error that can occur in this module
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Fail)]
pub enum ErrorKind {
    /// Tmp error to make ErrorKind implement Display, replace with first real error type
    #[fail(display = "tmp")]
    Tmp,
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
