#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

//! Loading and playing of audio files.
extern crate amethyst_assets;
extern crate amethyst_core;
extern crate cpal;
#[macro_use]
extern crate log;
extern crate rodio;
#[macro_use]
extern crate serde;
extern crate smallvec;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use self::{
    bundle::AudioBundle,
    components::*,
    formats::{AudioFormat, FlacFormat, Mp3Format, OggFormat, WavFormat},
    sink::AudioSink,
    source::{Source, SourceHandle},
    systems::*,
};

pub mod output;

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

mod bundle;
mod components;
mod end_signal;
mod formats;
mod sink;
mod source;
mod systems;

/// An error occurred while decoding the source.
#[derive(Debug)]
pub struct DecoderError;

impl Display for DecoderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str("DecoderError")
    }
}

impl Error for DecoderError {
    fn description(&self) -> &str {
        "An error occurred while decoding sound data."
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}
