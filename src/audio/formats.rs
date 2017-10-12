use rayon::ThreadPool;

use assets::*;

/// Loads audio from wav files.
pub struct WavFormat;

impl Format for WavFormat {
    const EXTENSIONS: &'static [&'static str] = &["wav"];
    type Data = Vec<u8>;
    type Error = NoError;
    type Result = Result<Self::Data, Self::Error>;

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        Ok(bytes)
    }
}

/// Loads audio from Ogg Vorbis files
pub struct OggFormat;

impl Format for OggFormat {
    const EXTENSIONS: &'static [&'static str] = &["ogg"];
    type Data = Vec<u8>;
    type Error = NoError;
    type Result = Result<Self::Data, Self::Error>;

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        Ok(bytes)
    }
}

/// Loads audio from Flac files.
pub struct FlacFormat;

impl Format for FlacFormat {
    const EXTENSIONS: &'static [&'static str] = &["flac"];
    type Data = Vec<u8>;
    type Error = NoError;
    type Result = Result<Self::Data, Self::Error>;

    fn parse(&self, bytes: Vec<u8>, _: &ThreadPool) -> Self::Result {
        Ok(bytes)
    }
}
