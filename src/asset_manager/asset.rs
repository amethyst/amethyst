use std::fmt::{Display, Error as FormatError, Formatter};

pub trait Asset {
    type Data: Into<Self>;
}

pub trait AssetFormat {
    fn file_extension() -> &'static str;
}

pub trait AssetStore {
    fn read_asset<F: AssetFormat>(&self, name: &str) -> Result<Box<[u8]>, String>;
}

#[derive(Debug)]
pub enum AssetStoreError {
    /// This asset does not exist in this asset
    /// store. Note that you must not add a file
    /// extension to `name` when accessing an asset.
    /// The file extension comes with the format
    /// parameter.
    NoSuchAsset,
    /// You do not have enough permissions to read
    /// this resource.
    PermissionDenied,
    /// There was a timeout when requesting to read this
    /// resource.
    Timeout,
    /// The asset store you tried to read from is not
    /// available. This may be the case if you tried
    /// to read from some server but it is offline.
    NotAvailable,
    /// Some error which does not match any of the above.
    Other(String),
}

impl Display for AssetStoreError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            AssetStoreError::NoSuchAsset => write!(f, "No such asset"),
            AssetStoreError::PermissionDenied => {
                write!(f, "You do not have enough permissions to access this asset")
            }
            AssetStoreError::Timeout => {
                write!(f, "A timeout occured when trying to read the asset")
            }
            AssetStoreError::NotAvailable => write!(f, "The asset storage could not be reached"),
            AssetStoreError::Other(ref x) => write!(f, "Othere error: {}", x),
        }
    }
}
