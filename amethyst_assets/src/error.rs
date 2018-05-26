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
    #[fail(display = "Could not get metadata of asset at \"{}\".", _0)]
    AssetMetadata(String),
    /// Failed to load asset data from a source
    #[fail(display = "Failed to load asset from source at \"{}\"", _0)]
    FetchAssetFromSource(String),
    /// Could not import an asset.
    #[fail(display = "Failed to import asset called \"{}\" of type \"{}\"", name, asset_type)]
    ImportAsset {
        /// The name of the asset
        name: String,
        /// The asset type
        asset_type: &'static str,
    },
    /// `SimpleFormat` got an error when converting raw bytes into an asset
    #[fail(display = "Failed to import asset called \"{}\" of type \"{}\" from its raw byte format",
           name, asset_type)]
    ImportAssetFromBytes {
        /// The name of the asset
        name: String,
        /// The asset type
        asset_type: &'static str,
    },
    /// There was an error reloading an asset
    #[fail(display = "Failed to reload asset called \"{}\" of type \"{}\".", name, asset_type)]
    Reload {
        /// The name of the asset
        name: String,
        /// The asset type
        asset_type: &'static str,
    },
    /// The single file reloader failed to reload a file with the given format .
    #[fail(display = "The single file reloader failed to reload a file at \"{}\" of type \"{}\".",
           path, asset_type)]
    SingleFileReload {
        /// The path to the asset
        path: String,
        /// The asset type
        asset_type: &'static str,
    },
    /// A custom asset dropper returned an error.
    #[fail(display = "A custom asset dropper returned an error.")]
    DropAsset,
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn err_msgs() {
        for (variant, msg) in vec![(
            ErrorKind::AssetMetadata("/some/path/or/other".into()),
            r#"Could not get metadata of asset at "/some/path/or/other"."#
        ), (
            ErrorKind::FetchAssetFromSource("/some/path/or/other".into()),
            r#"Failed to load asset from source at "/some/path/or/other""#
        ), (
            ErrorKind::ImportAsset {
                name: "name".into(),
                asset_type: "type"
            },
            r#"Failed to import asset called "name" of type "type""#
        ), (
            ErrorKind::ImportAssetFromBytes {
                name: "name".into(),
                asset_type: "type"
            },
            r#"Failed to import asset called "name" of type "type" from its raw byte format"#,
        ), (
            ErrorKind::Reload {
                name: "name".into(),
                asset_type: "type"
            },
            r#"Failed to reload asset called "name" of type "type"."#
        ), (
            ErrorKind::SingleFileReload {
                path: "/some/path/or/other".into(),
                asset_type: "type"
            },
            r#"The single file reloader failed to reload a file at "/some/path/or/other" of type "type"."#
        ), (
            ErrorKind::DropAsset,
            "A custom asset dropper returned an error."
        )] {
            assert_eq!(format!("{}", variant), msg)
        }

    }
}