use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use BoxedErr;

/// Error type returned when loading an asset.
/// Includes
///
/// * the `name` of the asset,
/// * the `format` identifier and
/// * and the error that occurred during loading.
#[derive(Debug)]
pub struct AssetError {
    /// The specifier of the asset which failed to load
    pub name: String,
    /// The format identifier.
    pub format: String,
    /// The error that's been raised.
    pub error: BoxedErr,
}

impl AssetError {
    pub(crate) fn new(name: String, format: String, error: BoxedErr) -> Self {
        AssetError {
            name,
            format,
            error,
        }
    }
}

impl Display for AssetError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "Failed to load asset {:?} of format {:?}: {}",
            &self.name,
            &self.format,
            &self.error
        )
    }
}

impl Error for AssetError {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.error)
    }
}
