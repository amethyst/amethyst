use std::sync::Arc;

use super::Source as Audio;
use assets::*;

pub struct AudioData(pub Vec<u8>);

/// Loads audio from wav files.
pub struct WavFormat;

impl Format<Audio> for WavFormat {
    const NAME: &'static str = "WAV";

    fn import(&self, name: String, source: Arc<Source>) -> Result<AudioData, BoxedErr> {
        source.load(&name).map(AudioData)
    }
}

/// Loads audio from Ogg Vorbis files
pub struct OggFormat;

impl Format<Audio> for OggFormat {
    const NAME: &'static str = "OGG";

    fn import(&self, name: String, source: Arc<Source>) -> Result<AudioData, BoxedErr> {
        source.load(&name).map(AudioData)
    }
}

/// Loads audio from Flac files.
pub struct FlacFormat;

impl Format<Audio> for FlacFormat {
    const NAME: &'static str = "FLAC";

    fn import(&self, name: String, source: Arc<Source>) -> Result<AudioData, BoxedErr> {
        source.load(&name).map(AudioData)
    }
}
