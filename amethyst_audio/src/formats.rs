use super::Source as Audio;
use amethyst_assets::*;

#[derive(Clone)]
pub struct AudioData(pub Vec<u8>);

/// Loads audio from wav files.
#[derive(Clone)]
pub struct WavFormat;

impl SimpleFormat<Audio> for WavFormat {
    const NAME: &'static str = "WAV";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<AudioData> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Ogg Vorbis files
#[derive(Clone)]
pub struct OggFormat;

impl SimpleFormat<Audio> for OggFormat {
    const NAME: &'static str = "OGG";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<AudioData> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Flac files.
#[derive(Clone)]
pub struct FlacFormat;

impl SimpleFormat<Audio> for FlacFormat {
    const NAME: &'static str = "FLAC";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> Result<AudioData> {
        Ok(AudioData(bytes))
    }
}
