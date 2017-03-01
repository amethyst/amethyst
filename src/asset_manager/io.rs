use std::fmt::{Display, Formatter, Error as FormatError};
use std::io::Error as IoError;
use std::str::Utf8Error;

/// Error type which may be raised when trying
/// to import some asset data.
#[derive(Debug)]
pub enum Error {
    /// The stream provided to the
    /// `Import` type is not in the right
    /// format.
    FormatError(String),
    /// There has been an IO Error when
    /// trying to read from the stream.
    IoError(IoError),
}

/// Import trait, which should be implemented together
/// with `AssetFormat` for each format that can be loaded
/// into asset data.
pub trait Import<T> {
    /// Imports `T` from a boxed slice of bytes.
    fn import(&self, bytes: Box<[u8]>) -> Result<T, Error>;
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::IoError(e)
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::FormatError("Invalid UTF8 character".to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &Error::FormatError(ref x) => {
                write!(f,
                       "Importing failed: There is an issue with the data format: {}",
                       x)
            }
            &Error::IoError(ref x) => write!(f, "Importing failed because of an IO error: {}", x),
        }
    }
}
