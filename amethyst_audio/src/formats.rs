use amethyst_assets::*;
use amethyst_error::Error;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct AudioData(pub Vec<u8>);
amethyst_assets::register_format_type!(AudioData);

/// Loads audio from wav files.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WavFormat;

amethyst_assets::register_format!("WAV", WavFormat as AudioData);
impl Format<AudioData> for WavFormat {
    fn name(&self) -> &'static str {
        "WAV"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Ogg Vorbis files
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct OggFormat;

amethyst_assets::register_format!("OGG", OggFormat as AudioData);
impl Format<AudioData> for OggFormat {
    fn name(&self) -> &'static str {
        "OGG"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from Flac files.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FlacFormat;

amethyst_assets::register_format!("FLAC", FlacFormat as AudioData);
impl Format<AudioData> for FlacFormat {
    fn name(&self) -> &'static str {
        "FLAC"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}

/// Loads audio from MP3 files.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Mp3Format;

amethyst_assets::register_format!("MP3", Mp3Format as AudioData);
impl Format<AudioData> for Mp3Format {
    fn name(&self) -> &'static str {
        "MP3"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<AudioData, Error> {
        Ok(AudioData(bytes))
    }
}
