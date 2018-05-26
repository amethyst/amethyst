//! Provides structures and functions used to get audio outputs.

// We have to use types from this to provide an output iterator type.
use std::fmt;
use std::io::Cursor;

use cpal::OutputDevices;
use failure::ResultExt;
use rodio::{default_output_device, output_devices, Decoder, Device, Sink, Source as RSource};

use source::Source;
use {Error, ErrorKind};

/// A speaker(s) through which audio can be played.
///
/// By convention, the default output is stored as a resource in the `World`.
#[derive(Clone, Eq, PartialEq)]
pub struct Output {
    pub(crate) device: Device,
}

impl Output {
    /// Gets the name of the output
    pub fn name(&self) -> String {
        self.device.name()
    }

    /// Play a sound once.  A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// This will return an Error if the loaded audio file in source could not be decoded.
    pub fn try_play_once(&self, source: &Source, volume: f32) -> Result<(), Error> {
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
    pub fn try_play_n_times(&self, source: &Source, volume: f32, n: u16) -> Result<(), Error> {
        let sink = Sink::new(&self.device);
        for _ in 0..n {
            sink.append(
                Decoder::new(Cursor::new(source.clone()))
                    .context(ErrorKind::Decoder)?
                    .amplify(volume),
            );
        }
        sink.detach();
        Ok(())
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Output")
            .field("device", &self.name())
            .finish()
    }
}

/// An iterator over outputs
pub struct OutputIterator {
    input: OutputDevices,
}

impl Iterator for OutputIterator {
    type Item = Output;

    fn next(&mut self) -> Option<Output> {
        self.input.next().map(|re| Output { device: re })
    }
}

/// Get the default output, returns none if no outputs are available.
pub fn default_output() -> Option<Output> {
    default_output_device().map(|re| Output { device: re })
}

/// Get a list of outputs available to the system.
pub fn outputs() -> OutputIterator {
    OutputIterator {
        input: output_devices(),
    }
}
