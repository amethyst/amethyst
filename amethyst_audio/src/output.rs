//! Provides structures and functions used to get audio outputs.

// We have to use types from this to provide an output iterator type.
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::io::Cursor;

use cpal::{default_endpoint, endpoints};
use cpal::EndpointsIterator;
use rodio::{Decoder, Endpoint, Sink, Source as RSource};

use super::DecoderError;
use super::source::Source;

/// A speaker(s) through which audio can be played.
///
/// By convention, the default output is stored as a resouce in the `World`.
pub struct Output {
    pub(crate) endpoint: Endpoint,
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
        let sink = Sink::new(&self.endpoint);
        match Decoder::new(Cursor::new(source.clone())) {
            Ok(source) => {
                sink.append(source.amplify(volume));
                sink.detach();
                Ok(())
            }

            // There is one and only one error that can be returned, which is unrecognized format
            // See documentation for DecoderError here:
            // https://docs.rs/rodio/0.5.1/rodio/decoder/enum.DecoderError.html
            Err(err) => {
                eprintln!("Error while playing sound: {:?}", err);
                Err(DecoderError)
            }
        }
    }

    /// Play a sound once. A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// This may silently fail, in order to get error information use `try_play_once`.
    pub fn play_once(&self, source: &Source, volume: f32) {
        if let Err(err) = self.try_play_once(source, volume) {
            eprintln!("An error occurred while trying to play a sound: {:?}", err);
        }
    }

    /// Play a sound n times. A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// This may silently fail, in order to get error information use `try_play_n_times`.
    pub fn play_n_times(&self, source: &Source, volume: f32, n: u16) {
        if let Err(err) = self.try_play_n_times(source, volume, n) {
            eprintln!("An error occurred while trying to play a sound: {:?}", err);
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
            sink.append(
                Decoder::new(Cursor::new(source.clone()))
                    .map_err(|_| DecoderError)?
                    .amplify(volume),
            );
        }
        sink.detach();
        Ok(())
    }
}

impl Debug for Output {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("Output { endpoint: ")?;
        formatter.write_str(self.name().as_str())?;
        formatter.write_str(" }")?;
        Ok(())
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
