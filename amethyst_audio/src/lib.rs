//! Loading and playing of audio files.
extern crate amethyst_assets;
extern crate amethyst_core;
extern crate cpal;
#[macro_use]
extern crate log;
extern crate rodio;
extern crate shred;
extern crate smallvec;
extern crate specs;

pub use self::bundle::AudioBundle;
pub use self::components::*;
pub use self::formats::{FlacFormat, OggFormat, WavFormat};
pub use self::sink::AudioSink;
pub use self::source::{Source, SourceHandle};
pub use self::systems::*;

pub mod output;

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

mod formats;
mod sink;
mod source;
mod components;
mod systems;
mod bundle;

/// An error occurred while decoding the source.
#[derive(Debug)]
pub struct DecoderError;

impl Display for DecoderError {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("DecoderError")
    }
}

impl Error for DecoderError {
    fn description(&self) -> &str {
        "An error occurred while decoding sound data."
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
