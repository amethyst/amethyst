//! Renderer error types.

use std::{error, fmt};

/// Common renderer error type.
#[derive(Debug)]
pub(crate) enum Error {
    /// Failed to parse a Spritesheet from RON.
    LoadSpritesheetError(ron::de::Error),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Error::*;

        match *self {
            LoadSpritesheetError(..) => write!(fmt, "Failed to parse SpriteSheet"),
        }
    }
}
