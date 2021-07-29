//! An audio module that enables basic audio output in Amethyst
//!
//! Based on [`rodio`], this crate provides audio output that works out of the box not only
//! on many mainstream platforms, but supports different backends (e.g. Alsa and Jack on Linux).
//!
//! [rodio]: https://docs.rs/rodio/0.14.0/rodio/index.html

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![deny(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility,
    clippy::all
)]
#![warn(clippy::pedantic)]
#![allow(clippy::new_without_default, clippy::module_name_repetitions)]

use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub use self::{
    bundle::AudioBundle,
    components::*,
    formats::{FlacFormat, Mp3Format, OggFormat, WavFormat},
    sink::Sink,
    source::{Source, SourceHandle},
    systems::*,
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

impl From<rodio::decoder::DecoderError> for DecoderError {
    fn from(_: rodio::decoder::DecoderError) -> Self {
        DecoderError
    }
}
