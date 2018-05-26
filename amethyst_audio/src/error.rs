use std::fmt;
use std::result::Result as StdResult;

use failure::{Backtrace, Context, Fail};

/// `std::result::Result` specialized to our error type.
pub type Result<T> = StdResult<T, Error>;

/// The audio subsystem error type
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// The different contexts for errors in the audio module
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Fail)]
pub enum ErrorKind {
    /// An error occurred decoding an audio asset
    #[fail(display = "An error occurred while decoding an audio asset")]
    Decoder,
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
    /// Get a reference to the `ErrorKind` of this error.
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
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
