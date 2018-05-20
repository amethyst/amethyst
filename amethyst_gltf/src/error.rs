use std::fmt;
use std::result::Result as StdResult;

use failure::{Backtrace, Context, Fail};
use gltf::json::{self, validation};

/// `std::result::Result` specialized to our error type.
pub type Result<T> = StdResult<T, Error>;

/// The gltf asset subsystem error type
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// The different contexts for errors in the gltf subsystem.
#[derive(Fail, Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// General error using the importer, that could be caused by various underlying errors. See
    /// the `cause()` for more info.
    #[fail(display = "Importer failed to import glTF data.")]
    Importer,
    /// GLTF have no default scene and the number of scenes is not 1
    #[fail(display = "GLTF have no default scene and the number of scenes is {}, not 1", _0)]
    InvalidNumberOfScenes(usize),
    /// GLTF primitive use a primitive type not support by gfx
    #[fail(display = "GLTF primitive type \"{}\" not support by gfx", _0)]
    PrimitiveMissingInGfx(String),
    /// GLTF primitive missing positions
    #[fail(display = "GLTF primitive missing positions")]
    MissingPositions,
    /// A loaded glTF buffer is not of the required length.
    #[fail(display = "A loaded glTF buffer is not of the required length for contents at \"{}\".",
           _0)]
    BufferLength(json::Path),
    /// A glTF extension required by the asset has not been enabled by the user.
    #[fail(display = "The required glTF extension \"{}\" has not been enabled by the user.", _0)]
    ExtensionDisabled(String),
    /// A glTF extension required by the asset is not supported by the library.
    #[fail(display = "The required glTF extension \"{}\" is not supported by the library.", _0)]
    ExtensionUnsupported(String),
    /// The glTF version of the asset is incompatible with the importer.
    #[fail(display = "The glTF version of the asset ({}) is incompatible with the importer.", _0)]
    IncompatibleVersion(String),
    /// Failure when deserializing .gltf or .glb JSON.
    #[fail(display = "Failure when deserializing .gltf or .glb JSON.")]
    MalformedJson,
    /// The .gltf data is invalid.
    // TODO I can't get this to work #[derivative(Hash="ignore")]
    #[fail(display = "The .gltf data is invalid: {:?}.", _0)]
    Validation(Vec<(json::Path, validation::Error)>),
    /// Unsupported/Unrecognised image type
    #[fail(display = "Image format with mime-type \"{}\" is unsupported/unrecognised", _0)]
    UnknownImageType(String),
    /// The renderer errored when creating a texture resource
    #[fail(display = "The renderer errored when creating a texture resource")]
    TextureCreation,
    /// Not implemented yet
    #[fail(display = "Not implemented yet")]
    NotImplemented,
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
