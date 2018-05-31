//! Provides structures used to load audio files.

use amethyst_assets::{Asset, Handle, ProcessingState, Result};
use amethyst_core::specs::prelude::VecStorage;

use formats::AudioData;

/// A handle to a source asset.
pub type SourceHandle = Handle<Source>;

/// A loaded audio file
#[derive(Clone)]
pub struct Source {
    /// The bytes of this audio source.
    pub bytes: Vec<u8>,
}

impl AsRef<[u8]> for Source {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Asset for Source {
    const NAME: &'static str = "audio::Source";
    type Data = AudioData;
    type HandleStorage = VecStorage<SourceHandle>;
}

impl Into<Result<ProcessingState<Source>>> for AudioData {
    fn into(self) -> Result<ProcessingState<Source>> {
        Ok(ProcessingState::Loaded(Source { bytes: self.0 }))
    }
}
