//! Provides audio formats

use std::io::Cursor;

use rayon::ThreadPool;
use rodio::Decoder;

use assets::*;
use audio::DecoderError;

pub struct RawAudioData {
    samples: Vec<i16>,
    sample_rate: u32,
    channels: u16,
}

type AudioFuture = SpawnedFuture<RawAudioData, DecoderError>;

fn decode_samples(bytes: Vec<u8>, pool: &ThreadPool) -> AudioFuture {
    AudioFuture::spawn(pool, move || {
        let decoder = Decoder::new(Cursor::new(bytes)).map_err(|_| DecoderError)?;
        let sample_rate = decoder.samples_rate();
        let channels = decoder.channels();
        RawAudioData {
            samples: decoder.collect::<Vec<i16>>(),
            sample_rate,
            channels,
        }
    })
}

/// Loads audio from wav files.
pub struct WavFormat;

impl Format for WavFormat {
    const EXTENSIONS: &'static [&'static str] = &["wav"];
    type Data = RawAudioData;
    type Error = DecoderError;
    type Result = AudioFuture;

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        decode_samples(bytes, pool)
    }
}

/// Loads audio from Ogg Vorbis files
pub struct OggFormat;

impl Format for OggFormat {
    const EXTENSIONS: &'static [&'static str] = &["ogg"];
    type Data = RawAudioData;
    type Error = DecoderError;
    type Result = AudioFuture;

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        decode_samples(bytes, pool)
    }
}

/// Loads audio from Flac files.
pub struct FlacFormat;

impl Format for FlacFormat {
    const EXTENSIONS: &'static [&'static str] = &["flac"];
    type Data = RawAudioData;
    type Error = DecoderError;
    type Result = AudioFuture;

    fn parse(&self, bytes: Vec<u8>, pool: &ThreadPool) -> Self::Result {
        decode_samples(bytes, pool)
    }
}
