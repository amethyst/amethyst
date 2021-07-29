//! Provides structures and functions used to get audio outputs.

// We have to use types from this to provide an output iterator type.
use std::fmt::{Debug, Formatter, Result as FmtResult};

use log::error;
pub use rodio::OutputStream;
use rodio::{Device, DeviceTrait, OutputStreamHandle, PlayError, StreamError};

use crate::{sink::Sink, source::Source, DecoderError};

/// An audio output that can be used to play audio directly,
/// or to spawn `Sink`s that are more flexible.
pub struct Output {
    /// Name of the output device being used
    pub name: String,
    /// Handle to an `OutputStream` that can be shared across threads
    pub stream_handle: OutputStreamHandle,
}

impl Output {
    /// Spawn a new Sink (audio input).
    pub fn try_spawn_sink(&self) -> Result<Sink, PlayError> {
        Sink::try_new(&self.stream_handle)
    }

    /// Play a sound once.  A volume of 1.0 is unchanged, while 0.0 is silent.
    ///
    /// # Errors
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
    /// # Errors
    /// This will return an Error if the loaded audio file in source could not be decoded.
    /// # Panics
    /// Blah blah
    pub fn try_play_n_times(
        &self,
        source: &Source,
        volume: f32,
        n: u16,
    ) -> Result<(), DecoderError> {
        let sink = self.try_spawn_sink().unwrap();

        for _ in 0..n {
            sink.append(source, volume)?;
        }
        sink.detach();

        Ok(())
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Output")
            .field("device", &self.name)
            .finish()
    }
}

/*
/// An iterator over outputs
#[allow(missing_debug_implementations)]
pub struct OutputIterator {
    devices: OutputDevices<Devices>,
}

impl Iterator for OutputIterator {
    type Item = Output;

    fn next(&mut self) -> Option<Output> {
        self.devices.next().map(|device| Output {
            device: Arc::new(device),
        })
    }
}

/// Get a list of outputs available to the system.
///
/// # Panics
///
/// Panics if the system does not support audio output and hence no output devices
/// are found.
#[must_use]
pub fn outputs() -> OutputIterator {
    let devices = cpal::default_host()
        .output_devices()
        .unwrap_or_else(|e| panic!("Error retrieving output devices: `{}`", e));
    OutputIterator { devices }
}
*/

/// Initialize default output
///
/// # Errors
///
/// The result is a `StreamError` if initializing the `OutputStream` from default
/// device has failed.
pub fn init_output() -> Result<(OutputStream, Output), StreamError> {
    let (stream, stream_handle) = OutputStream::try_default()?;

    let output = Output {
        name: String::from("default"),
        stream_handle,
    };

    Ok((stream, output))
}

/// Initialize default output
///
/// # Errors
///
/// The result is a `StreamError` if initializing the `OutputStream` from the specified
/// device has failed.
pub fn init_output_from_device(device: &Device) -> Result<(OutputStream, Output), StreamError> {
    let (stream, stream_handle) = OutputStream::try_from_device(device)?;

    let output = Output {
        name: device
            .name()
            .unwrap_or_else(|_| String::from("Unknown device")),
        stream_handle,
    };

    Ok((stream, output))
}

#[cfg(test)]
#[cfg(target_os = "linux")] // these tests only work in linux CI
mod tests {
    use std::{fs::File, io::Read, vec::Vec};

    use crate::{output::init_output, source::Source, DecoderError};
    use amethyst_utils::app_root_dir::application_root_dir;
    use rodio::cpal::{default_host, traits::HostTrait};

    #[test]
    fn test_play_wav() {
        test_play("tests/sound_test.wav", true)
    }

    #[test]
    fn test_play_mp3() {
        test_play("tests/sound_test.mp3", true);
    }

    #[test]
    fn test_play_flac() {
        test_play("tests/sound_test.flac", true);
    }

    #[test]
    fn test_play_ogg() {
        test_play("tests/sound_test.ogg", true);
    }

    #[test]
    fn test_play_fake() {
        test_play("tests/sound_test.fake", false);
    }

    // test_play tests the play APIs for Output
    fn test_play(file_name: &str, should_pass: bool) {
        // Get the full file path
        let app_root = application_root_dir().unwrap();
        let audio_path = app_root.join(file_name);

        // Convert the file contents into a byte vec
        let mut f = File::open(audio_path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();

        // Create a Source from those bytes
        let src = Source { bytes: buffer };

        // Set volume and number of times to play
        let vol: f32 = 1.0;
        let n: u16 = 5;

        // Test each of the play APIs
        let (_stream, output) = init_output().unwrap();

        output.play_once(&src, vol);

        output.play_n_times(&src, vol, n);

        let result_try_play_once = output.try_play_once(&src, vol);
        check_result(result_try_play_once, should_pass);

        let result_try_play_n_times = output.try_play_n_times(&src, vol, n);
        check_result(result_try_play_n_times, should_pass);
    }

    fn check_result(result: Result<(), DecoderError>, should_pass: bool) {
        match result {
            Ok(_pass) => {
                assert!(
                    should_pass,
                    "Expected `play` result to be Err(..), but was Ok(..)"
                )
            }
            Err(fail) => {
                assert!(
                    !should_pass,
                    "Expected `play` result to be `Ok(..)`, but was {:?}",
                    fail
                )
            }
        };
    }

    #[test]
    fn output_devices() {
        let mut dev: bool = false;
        if default_host().default_output_device().is_some() {
            dev = true;
        }
        assert!(dev);
    }
}
