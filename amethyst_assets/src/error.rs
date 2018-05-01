use std::fmt;
use std::result::Result as StdResult;

use failure::{Backtrace, Context, Fail};

/// `std::result::Result` specialized to our error type.
pub type Result<T> = StdResult<T, Error>;

/// The assets subsystem error type
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// The different contexts for errors in the assets module
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Fail)]
pub enum ErrorKind {
    /// Error getting the metadata of an asset on disk
    #[fail(display="Could not get metadata of asset at \"{}\".", _0)]
    AssetMetadata(String),
    /// Returned if an asset with a given name failed to load.
    #[fail(display="Failed to load asset from disk at \"{}\"", _0)]
    FetchAssetFromDisk(String),
    /// Could not import an asset.
    #[fail(display="Failed to import asset called \"{}\" of type \"{}\"", name, asset_type)]
    ImportAsset {
    /// The name of the asset
        name: String,
    /// The asset type
        asset_type: &'static str,
    },
    /// `SimpleFormat` got an error when converting raw bytes into an asset
    #[fail(display="Failed to import asset called \"{}\" of type \"{}\" from its raw byte format", name, asset_type)]
    ImportAssetFromBytes {
        /// The name of the asset
        name: String,
        /// The asset type
        asset_type: &'static str,
    },
    /// There was an error reloading an asset
    #[fail(display="Failed to reload asset called \"{}\" of type \"{}\".", name, asset_type)]
    Reload {
    /// The name of the asset
        name: String,
    /// The asset type
        asset_type: &'static str,
    },
    /// The single file reloader failed to reload a file with the given format .
    #[fail(display="The single file reloader failed to reload a file in \"{}\" format.", _0)]
    SingleFileReload {
        /// The path to the asset
        path: String,
        /// The asset type
        asset_type: &'static str,
    },
    /// A custom asset dropper returned an error.
    #[fail(display="A custom asset dropper returned an error.")]
    DropAsset,
    ///// Returned if a source could not retrieve something.
    //#[fail(display="Failed to load bytes from source")]
    //Source,
    ///// Returned if a format failed to load the asset data.
    //#[fail(display="Format \"{}\" could not load asset", _0)]
    //Format(&'static str),
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
