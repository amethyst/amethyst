//! Renderer error types.

use std::{error, fmt};

/// Common renderer error type.
#[derive(Debug)]
pub(crate) enum Error {
    /// A render target with the given name does not exist.
    NoSuchTarget(String),
    /// Failed to create a pipeline state object (PSO).
    ProgramCreation,
    /// Failed to interact with the ECS.
    PixelDataMismatch(String),
    /// The window handle associated with the renderer has been destroyed.
    WindowDestroyed,
    /// Failed to parse a Spritesheet from RON.
    LoadSpritesheetError(ron::de::Error),
    /// Failed to build texture.
    BuildTextureError,
    /// Unsupported texture size.
    UnsupportedTextureSize(u32, u32),
    /// Image decoding error.
    DecodeImageError,
    /// Failed to create texture.
    CreateTextureError,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Error::*;

        match *self {
            NoSuchTarget(ref e) => write!(fmt, "Nonexistent target: {}", e),
            ProgramCreation => write!(fmt, "Program compilation failed"),
            PixelDataMismatch(ref e) => write!(fmt, "Pixel data and metadata do not match: {}", e),
            WindowDestroyed => write!(fmt, "Window has been destroyed"),
            LoadSpritesheetError(ref e) => write!(fmt, "Failed to parse SpriteSheet: {}", e),
            BuildTextureError => write!(fmt, "Failed to build texture"),
            UnsupportedTextureSize(w, h) => write!(
                fmt,
                "Unsupported texture size (expected: ({}, {}), got: ({}, {})",
                u16::max_value(),
                u16::max_value(),
                w,
                h,
            ),
            DecodeImageError => write!(fmt, "Image decoding failed"),
            CreateTextureError => write!(fmt, "Failed to create texture from texture data"),
        }
    }
}
