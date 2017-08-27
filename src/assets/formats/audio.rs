//! Provides audio formats

use assets::*;
use rayon::ThreadPool;

/// Loads audio from wav files.
pub struct WavFormat;

impl Format for WavFormat {
    type Data = Vec<u8>;
    type Error = NoError;
    type Result = Result<Self::Data, Self::Error>;

    fn extension() -> &'static str {
        "wav"
    }

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        Ok(bytes)
    }
}

/// Loads audio from Ogg Vorbis files
pub struct OggFormat;

impl Format for OggFormat {
    type Data = Vec<u8>;
    type Error = NoError;
    type Result = Result<Self::Data, Self::Error>;

    fn extension() -> &'static str {
        "ogg"
    }

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        Ok(bytes)
    }
}

/// Loads audio from Flac files.
pub struct FlacFormat;

impl Format for FlacFormat {
    type Data = Vec<u8>;
    type Error = NoError;
    type Result = Result<Self::Data, Self::Error>;

    fn extension() -> &'static str {
        "flac"
    }

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        Ok(bytes)
    }
}
