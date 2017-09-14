//! Provides structures used to load audio files.

use std::sync::Arc;

use super::AudioContext;
use assets::*;

use rodio::buffer::SamplesBuffer;

/// A loaded audio file
#[derive(Clone)]
pub struct Source {
    pub(crate) pointer: AssetPtr<Arc<SamplesBuffer<i16>>, Source>,
}

impl Asset for Source {
    type Context = AudioContext;
}
