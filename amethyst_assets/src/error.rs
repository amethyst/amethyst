use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use futures::future::SharedError;

use asset::AssetSpec;

/// Error type returned when loading an asset.
/// Includes the `AssetSpec` and the error (`LoadError`).
#[derive(Debug)]
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
where
    A: Display,
    F: Display,
    S: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "Failed to load asset \"{}\" of format \"{:?}\" from storage with id \"{}\": {}",
            &self.asset.name,
            &self.asset.exts,
            &self.asset.store.id(),
            &self.error
        )
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
where
    A: Display,
    F: Display,
    S: Display,
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
where
    A: Error,
    F: Error,
    S: Error,
{
    fn description(&self) -> &str {
        "Failed to load asset"
    }

    fn cause(&self) -> Option<&Error> {
        Some(&self.error)
    }
}

impl<A, F, S> Error for LoadError<A, F, S>
where
    A: Error,
    F: Error,
    S: Error,
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


/// Shared version of error
pub struct SharedAssetError<E>(SharedError<E>);

impl<E> AsRef<E> for SharedAssetError<E> {
    fn as_ref(&self) -> &E {
        &*self.0
    }
}

impl<E> Error for SharedAssetError<E>
where
    E: Error,
{
    fn description(&self) -> &str {
        self.as_ref().description()
    }

    fn cause(&self) -> Option<&Error> {
        self.as_ref().cause()
    }
}

impl<E> Debug for SharedAssetError<E>
where
    E: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.as_ref().fmt(f)
    }
}

impl<E> Display for SharedAssetError<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.as_ref().fmt(f)
    }
}

impl<E> From<SharedError<E>> for SharedAssetError<E> {
    fn from(err: SharedError<E>) -> Self {
        SharedAssetError(err)
    }
}
