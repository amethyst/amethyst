//! Renderer error types.

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as StdResult,
};

/// Renderer result type.
pub type Result<T> = StdResult<T, Error>;

/// Common renderer error type.
#[derive(Debug)]
pub enum Error {
    /// Failed to create a buffer.
    BufferCreation(gfx::buffer::CreationError),
    /// A render target with the given name does not exist.
    NoSuchTarget(String),
    /// Failed to initialize a render pass.
    PassInit(gfx::PipelineStateError<String>),
    /// Failed to create a pipeline state object (PSO).
    PipelineCreation(gfx_core::pso::CreationError),
    /// Failed to create thread pool.
    PoolCreation(String),
    /// Failed to create and link a shader program.
    ProgramCreation(gfx::shade::ProgramError),
    /// Failed to create a resource view.
    ResViewCreation(gfx::ResourceViewError),
    /// Failed to interact with the ECS.
    SpecsError(amethyst_core::specs::error::Error),
    /// Failed to create a render target.
    TargetCreation(gfx::CombinedError),
    /// Failed to create a texture resource.
    TextureCreation(gfx::texture::CreationError),
    /// The given pixel data and metadata do not match.
    PixelDataMismatch(String),
    /// The window handle associated with the renderer has been destroyed.
    WindowDestroyed,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BufferCreation(_) => "Failed to create buffer!",
            Error::NoSuchTarget(_) => "Target with this name does not exist!",
            Error::PassInit(_) => "Failed to initialize render pass!",
            Error::PipelineCreation(_) => "Failed to create PSO!",
            Error::PoolCreation(_) => "Failed to create thread pool!",
            Error::ProgramCreation(_) => "Failed to create shader program!",
            Error::ResViewCreation(_) => "Failed to create resource view!",
            Error::SpecsError(_) => "Failed to interact with the ECS!",
            Error::TargetCreation(_) => "Failed to create render target!",
            Error::TextureCreation(_) => "Failed to create texture!",
            Error::PixelDataMismatch(_) => "Pixel data and metadata do not match!",
            Error::WindowDestroyed => "Window has been destroyed!",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Error::BufferCreation(ref e) => Some(e),
            Error::PassInit(ref e) => Some(e),
            Error::PipelineCreation(ref e) => Some(e),
            Error::ProgramCreation(ref e) => Some(e),
            Error::ResViewCreation(ref e) => Some(e),
            Error::SpecsError(ref e) => Some(e),
            Error::TargetCreation(ref e) => Some(e),
            Error::TextureCreation(ref e) => Some(e),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Error::BufferCreation(ref e) => write!(fmt, "Buffer creation failed: {}", e),
            Error::NoSuchTarget(ref e) => write!(fmt, "Nonexistent target: {}", e),
            Error::PassInit(ref e) => write!(fmt, "Pass initialization failed: {}", e),
            Error::PipelineCreation(ref e) => write!(fmt, "PSO creation failed: {}", e),
            Error::PoolCreation(ref e) => write!(fmt, "Thread pool creation failed: {}", e),
            Error::ProgramCreation(ref e) => write!(fmt, "Program compilation failed: {}", e),
            Error::ResViewCreation(ref e) => write!(fmt, "Resource view creation failed: {}", e),
            Error::SpecsError(ref e) => write!(fmt, "Interaction with ECS failed: {}", e),
            Error::TargetCreation(ref e) => write!(fmt, "Target creation failed: {}", e),
            Error::TextureCreation(ref e) => write!(fmt, "Texture creation failed: {}", e),
            Error::PixelDataMismatch(ref e) => {
                write!(fmt, "Pixel data and metadata do not match: {}", e)
            }
            Error::WindowDestroyed => write!(fmt, "Window has been destroyed"),
        }
    }
}

impl From<gfx::CombinedError> for Error {
    fn from(e: gfx::CombinedError) -> Error {
        Error::TargetCreation(e)
    }
}

impl From<gfx::PipelineStateError<String>> for Error {
    fn from(e: gfx::PipelineStateError<String>) -> Error {
        Error::PassInit(e)
    }
}

impl From<gfx::ResourceViewError> for Error {
    fn from(e: gfx::ResourceViewError) -> Error {
        Error::ResViewCreation(e)
    }
}

impl From<gfx::buffer::CreationError> for Error {
    fn from(e: gfx::buffer::CreationError) -> Error {
        Error::BufferCreation(e)
    }
}

impl From<gfx::shade::ProgramError> for Error {
    fn from(e: gfx::shade::ProgramError) -> Error {
        Error::ProgramCreation(e)
    }
}

impl From<gfx::texture::CreationError> for Error {
    fn from(e: gfx::texture::CreationError) -> Error {
        Error::TextureCreation(e)
    }
}

impl From<gfx_core::pso::CreationError> for Error {
    fn from(e: gfx_core::pso::CreationError) -> Error {
        Error::PipelineCreation(e)
    }
}

impl From<amethyst_core::specs::error::Error> for Error {
    fn from(e: amethyst_core::specs::error::Error) -> Error {
        Error::SpecsError(e)
    }
}
