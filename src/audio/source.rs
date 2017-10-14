//! Provides structures used to load audio files.

use assets::{Asset, BoxedErr};

use audio::formats::AudioData;

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
}

impl Into<Result<Source, BoxedErr>> for AudioData {
    fn into(self) -> Result<Source, BoxedErr> {
        Ok(Source { bytes: self.0 })
    }
}
