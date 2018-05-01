//! Renderer error types.

use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::result::Result as StdResult;

use failure::{Error as FailureError, Backtrace, Context, Fail};
use gfx;
use gfx_core;

/// Renderer result type.
pub type Result<T> = StdResult<T, Error>;

/// The renderer subsystem error type
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

/// Common renderer error type.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Fail)]
pub enum ErrorKind {
    /// Failed to create a buffer.
    #[fail(display = "Failed to create buffer.")]
    BufferCreation,
    /// A render target with the given name does not exist.
    #[fail(display = "No render target with name \"{}\" exists.", _0)]
    NoSuchTarget(String),
    /// Failed to initialize a render pass.
    #[fail(display = "Could not initialize a render pass.")]
    PassInit,
    /// Failed to create a pipeline state object (PSO).
    #[fail(display = "Failed to create a pipeline state object (PSO).")]
    PipelineCreation,
    /// Failed to create thread pool.
    #[fail(display = "Failed to create thread pool.")]
    PoolCreation,
    /// Failed to create and link a shader program.
    #[fail(display = "Failed to create and link a shader program.")]
    ProgramCreation,
    /// Failed to create a resource view.
    #[fail(display = "Failed to create a resource view.")]
    ResViewCreation,
    /// Failed to create a render target.
    #[fail(display = "Failed to create a render target.")]
    TargetCreation,
    /// Failed to create a mesh resource.
    #[fail(display = "Failed to create a mesh resource.")]
    MeshCreation,
    /// Failed to create a texture resource.
    #[fail(display = "Failed to create a texture resource.")]
    TextureCreation,
    /// The window handle associated with the renderer has been destroyed.
    #[fail(display = "The window handle associated with the renderer has been destroyed.")]
    WindowDestroyed,
    /// An image decoder failed to decode an image.
    #[fail(display = "An image decoder failed to decode an image.")]
    DecodeImage,
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
    /// Get the kind of this error.
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

impl From<gfx::CombinedError> for Error {
    fn from(e: gfx::CombinedError) -> Error {
        e.context(ErrorKind::TargetCreation).into()
    }
}

impl From<gfx::PipelineStateError<String>> for Error {
    fn from(e: gfx::PipelineStateError<String>) -> Error {
        e.context(ErrorKind::PassInit).into()
    }
}

impl From<gfx::ResourceViewError> for Error {
    fn from(e: gfx::ResourceViewError) -> Error {
        e.context(ErrorKind::ResViewCreation).into()
    }
}

impl From<gfx::buffer::CreationError> for Error {
    fn from(e: gfx::buffer::CreationError) -> Error {
        e.context(ErrorKind::BufferCreation).into()
    }
}

impl From<gfx::shade::ProgramError> for Error {
    fn from(e: gfx::shade::ProgramError) -> Error {
        e.context(ErrorKind::ProgramCreation).into()
    }
}

impl From<gfx::texture::CreationError> for Error {
    fn from(e: gfx::texture::CreationError) -> Error {
        e.context(ErrorKind::TextureCreation).into()
    }
}

impl From<gfx_core::pso::CreationError> for Error {
    fn from(e: gfx_core::pso::CreationError) -> Error {
        e.context(ErrorKind::PipelineCreation).into()
    }
}
