//! Provides structures used to load audio files.

use amethyst_assets::{Asset, Handle, Result};
use specs::VecStorage;

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

impl Into<Result<Source>> for AudioData {
    fn into(self) -> Result<Source> {
        Ok(Source { bytes: self.0 })
    }
}
