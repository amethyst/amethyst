use std::fmt;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use failure::{Backtrace, Context, Fail};

/// `std::result::Result` specialized to our error type.
pub type Result<T> = StdResult<T, Error>;

/// The assets subsystem error type
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// Error related to anything that manages/creates configurations as well as
/// "workspace"-related things.
#[derive(Debug, Clone, Fail, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ErrorKind {
    /// Forward to the `std::io::Error` error.
    #[fail(display = "Error reading the config file from disk.")]
    File,
    /// Errors related to serde's parsing of configuration files.
    #[fail(display = "Error parsing config file.")]
    Parser,
    /// Occurs if a value is ill-formed during serialization (like a poisoned mutex).
    #[fail(display = "Error serializing config file.")]
    Serializer,
    /// Currently, the config file extension must be `.ron`.
    #[fail(display = "Config file has wrong extension, expected \".ron\", found {}.", _0)]
    Extension(WrongExtension),
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

    /// Helper function to wrap the `PathBuf` when we create the `Extension` variant.
    pub fn wrong_extension(path: PathBuf) -> Self {
        ErrorKind::Extension(WrongExtension { inner: path }).into()
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
/// Little helper struct to make displaying extensions work
#[derive(Debug, Clone, Fail, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WrongExtension {
    /// The path to the config file
    pub inner: PathBuf,
}

impl WrongExtension {
    /// Unwrap the inner `PathBuf`
    pub fn into_path_buf(self) -> PathBuf {
        self.inner
    }
}

impl Deref for WrongExtension {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.inner
    }
}

impl fmt::Display for WrongExtension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner.extension() {
            Some(ext) => write!(f, "\".{:?}\"", ext),
            None => write!(f, "a directory"),
        }
    }
}
