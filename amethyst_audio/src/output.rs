//! Provides structures and functions used to get audio outputs.

// We have to use types from this to provide an output iterator type.
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::io::Cursor;
use std::sync::atomic::{AtomicIsize, Ordering, ATOMIC_ISIZE_INIT};

use cpal::{default_endpoint, endpoints};
use cpal::EndpointsIterator;
use rodio::{Decoder, Endpoint, Sink, Source as RSource};

use DecoderError;
use end_signal::EndSignalSource;
use source::Source;

// These are isize values because due to thread interactions it is possible, however unlikely, that
// SOUNDS_PLAYING may be temporarily < 0.  If this happens, it should be resolved very quickly by
// the threads completing their instructions.
pub(crate) const MAX_SOUNDS_PLAYING: isize = 300;
pub(crate) static SOUNDS_PLAYING: AtomicIsize = ATOMIC_ISIZE_INIT;

/// A speaker(s) through which audio can be played.
///
/// By convention, the default output is stored as a resource in the `World`.
#[derive(Clone)]
pub struct Output {
    pub(crate) endpoint: Endpoint,
}

impl Eq for Output {}

impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint == other.endpoint
    }
}

impl Output {
    /// Gets the name of the output
    pub fn name(&self) -> String {
        self.endpoint.name()
    }

    /// Play a sound once.  A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// This will return an Error if the loaded audio file in source could not be decoded.
    pub fn try_play_once(&self, source: &Source, volume: f32) -> Result<(), DecoderError> {
        self.try_play_n_times(source, volume, 1)
    }

    /// Play a sound once. A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// This may silently fail, in order to get error information use `try_play_once`.
    pub fn play_once(&self, source: &Source, volume: f32) {
        self.play_n_times(source, volume, 1);
    }

    /// Play a sound n times. A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// This may silently fail, in order to get error information use `try_play_n_times`.
    pub fn play_n_times(&self, source: &Source, volume: f32, n: u16) {
        if let Err(err) = self.try_play_n_times(source, volume, n) {
            error!("An error occurred while trying to play a sound: {:?}", err);
        }
    }

    /// Play a sound n times. A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// This will return an Error if the loaded audio file in source could not be decoded.
    pub fn try_play_n_times(
        &self,
        source: &Source,
        volume: f32,
        n: u16,
    ) -> Result<(), DecoderError> {
        let sink = Sink::new(&self.endpoint);
        for _ in 0..n {
            if SOUNDS_PLAYING.load(Ordering::Relaxed) < MAX_SOUNDS_PLAYING {
                sink.append(EndSignalSource::new(
                    Decoder::new(Cursor::new(source.clone()))
                        .map_err(|_| DecoderError)?
                        .amplify(volume),
                    || {
                        SOUNDS_PLAYING.fetch_sub(1, Ordering::Relaxed);
                    },
                ));
                SOUNDS_PLAYING.fetch_add(1, Ordering::Relaxed);
            }
        }
        sink.detach();
        Ok(())
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Output")
            .field("endpoint", &self.name())
            .finish()
    }
}

/// An iterator over outputs
pub struct OutputIterator {
    input: EndpointsIterator,
}

impl Iterator for OutputIterator {
    type Item = Output;

    fn next(&mut self) -> Option<Output> {
        self.input.next().map(|re| Output { endpoint: re })
    }
}

/// Get the default output, returns none if no outputs are available.
pub fn default_output() -> Option<Output> {
    default_endpoint().map(|re| Output { endpoint: re })
}

/// Get a list of outputs available to the system.
pub fn outputs() -> OutputIterator {
    OutputIterator { input: endpoints() }
}
