//! Provides structures used to load audio files.

use amethyst_assets::{Asset, BoxedErr, Handle};
use specs::DenseVecStorage;

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
    type Data = AudioData;
    type HandleStorage = DenseVecStorage<SourceHandle>;
}

impl Into<Result<Source, BoxedErr>> for AudioData {
    fn into(self) -> Result<Source, BoxedErr> {
        Ok(Source { bytes: self.0 })
    }
}
