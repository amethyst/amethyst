use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use asset::AssetSpec;

/// Error type returned when loading an asset.
/// Includes the `AssetSpec` and the error (`LoadError`).
#[derive(Clone, Debug)]
pub struct AssetError<A, F, S> {
    /// The specifier of the asset which failed to load
    pub asset: AssetSpec,
    /// The error that's been raised.
    pub error: LoadError<A, F, S>,
}

impl<A, F, S> AssetError<A, F, S> {
    pub(crate) fn new(asset: AssetSpec, error: LoadError<A, F, S>) -> Self {
        AssetError { asset, error }
    }
}

impl<A, F, S> Display for AssetError<A, F, S>
    where A: Display,
          F: Display,
          S: Display
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f,
               "Failed to load asset \"{}\" of format \"{}\" from storage with id \"{}\": {}",
               &self.asset.name,
               &self.asset.ext,
               &self.asset.store.id(),
               &self.error)
    }
}

/// Combined error type which is produced when loading an
/// asset. This error does not include information which asset
/// failed to load. For that, please look at `AssetError`.
#[derive(Clone, Debug)]
pub enum LoadError<A, F, S> {
    /// The conversion from data -> asset failed.
    AssetError(A),
    /// The conversion from bytes -> data failed.
    FormatError(F),
    /// The storage was unable to retrieve the requested data.
    StorageError(S),
}

impl<A, F, S> Display for LoadError<A, F, S>
    where A: Display,
          F: Display,
          S: Display
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            LoadError::AssetError(ref e) => write!(f, "Failed to load asset: {}", e),
            LoadError::FormatError(ref e) => write!(f, "Failed to load data: {}", e),
            LoadError::StorageError(ref e) => write!(f, "Failed to load from storage: {}", e),
        }
    }
}

impl<A, F, S> Error for AssetError<A, F, S>
    where A: Error,
          F: Error,
          S: Error
{
    fn description(&self) -> &str {
        "Failed to load asset"
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.error)
    }
}

impl<A, F, S> Error for LoadError<A, F, S>
    where A: Error,
          F: Error,
          S: Error
{
    fn cause(&self) -> Option<&Error> {
        let cause: &Error = match *self {
            LoadError::AssetError(ref e) => e,
            LoadError::FormatError(ref e) => e,
            LoadError::StorageError(ref e) => e,
        };

        Some(cause)
    }

    fn description(&self) -> &str {
        match *self {
            LoadError::AssetError(_) => "Failed to load asset",
            LoadError::FormatError(_) => "Failed to load data",
            LoadError::StorageError(_) => "Failed to load from storage",
        }
    }
}

/// An error type which cannot be instantiated.
/// Used as a placeholder for associated error types if
/// something cannot fail.
#[derive(Debug)]
pub enum NoError {}

impl Display for NoError {
    fn fmt(&self, _: &mut Formatter) -> FmtResult {
        match *self {}
    }
}

impl Error for NoError {
    fn description(&self) -> &str {
        match *self {}
    }
}
