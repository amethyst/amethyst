//! Provides a context for loading a file.

use std::sync::Arc;

use rayon::ThreadPool;
use rodio::buffer::SamplesBuffer;

use super::DecoderError;
use super::source::Source;
use assets::*;

/// A context for loading audio files
pub struct AudioContext {
    cache: Cache<AssetFuture<Source>>,
}

impl AudioContext {
    /// Creates a new audio context.
    pub fn new() -> AudioContext {
        AudioContext { cache: Cache::new() }
    }
}

impl Context for AudioContext {
    type Asset = Source;
    type Data = RawAudioData;
    type Error = NoError;
    type Result = Result<Self::Asset, Self::Error>;

    fn category(&self) -> &str {
        "audio"
    }

    fn create_asset(&self, data: Self::Data, _: &ThreadPool) -> Result<Source, NoError> {
        Ok(Source { pointer: AssetPtr::new(Arc::new(SamplesBuffer::new(data.channels, data.sample_rate, data.samples))) })
    }

    fn update(&self, spec: &AssetSpec, asset: AssetFuture<Source>) {
        if let Some(updated) = self.cache
            .access(spec, |a| match a.peek() {
                Some(Ok(a)) => {
                    (*a).pointer.push_update(asset);
                    None
                }
                _ => Some(asset),
            })
            .and_then(|a| a)
        {
            self.cache.insert(spec.clone(), updated);
        }
    }
}
