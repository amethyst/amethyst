//! Loading and playing of audio files.
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

#[doc(no_inline)]
pub use self::{
    bundle::AudioBundle,
    components::*,
    formats::{FlacFormat, Mp3Format, OggFormat, WavFormat},
    sink::AudioSink,
    source::{Source, SourceHandle},
    systems::*,
};

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub mod output;

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
