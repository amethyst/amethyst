//! Provides structures and functions used to get audio outputs.

// We have to use types from this to provide an output iterator type.
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    io::Cursor,
};

use cpal::OutputDevices;
use log::error;
use rodio::{default_output_device, output_devices, Decoder, Device, Sink, Source as RSource};

use amethyst_core::shred::Resources;

use crate::{sink::AudioSink, source::Source, DecoderError};

/// A speaker(s) through which audio can be played.
///
/// By convention, the default output is stored as a resource in the `World`.
#[derive(Clone, Eq, PartialEq)]
pub struct Output {
    pub(crate) device: Device,
}

/// Convenience method for opening the default output device.
///
/// Since most modern hardware features audio output, this implementation fails if a device can't
/// be initialized. Use an alternative initialization scheme if running on hardware without an
/// integrated audio chip.
impl Default for Output {
    fn default() -> Self {
        default_output_device()
            .map(|re| Output { device: re })
            .expect("No default output device")
    }
}

impl Output {
    /// Gets the name of the output
    pub fn name(&self) -> String {
        self.device.name()
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
        let sink = Sink::new(&self.device);
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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

/// Initialize default output
pub fn init_output(res: &mut Resources) {
    if let Some(o) = default_output() {
        res.entry::<AudioSink>()
            .or_insert_with(|| AudioSink::new(&o));
        res.entry::<Output>().or_insert_with(|| o);
    } else {
        error!("Failed finding a default audio output to hook AudioSink to, audio will not work!")
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, vec::Vec};

    use amethyst_utils::app_root_dir::application_root_dir;

    use crate::{output::Output, source::Source, DecoderError};

    #[test]
    #[cfg(target_os = "linux")]
    fn test_play_wav() {
        test_play("tests/sound_test.wav", true)
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_play_mp3() {
        test_play("tests/sound_test.mp3", true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_play_flac() {
        test_play("tests/sound_test.flac", true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_play_ogg() {
        test_play("tests/sound_test.ogg", true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_play_fake() {
        test_play("tests/sound_test.fake", false);
    }

    // test_play tests the play APIs for Output
    #[cfg(target_os = "linux")]
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
        let vol: f32 = 4.0;
        let n: u16 = 5;

        // Test each of the play APIs
        let output = Output::default();

        output.play_once(&src, vol);

        output.play_n_times(&src, vol, n);

        let result_try_play_once = output.try_play_once(&src, vol);
        check_result(result_try_play_once, should_pass);

        let result_try_play_n_times = output.try_play_n_times(&src, vol, n);
        check_result(result_try_play_n_times, should_pass);
    }

    #[cfg(target_os = "linux")]
    fn check_result(result: Result<(), DecoderError>, should_pass: bool) {
        match result {
            Ok(_pass) => assert!(
                should_pass,
                "Expected `play` result to be Err(..), but was Ok(..)"
            ),
            Err(fail) => assert!(
                !should_pass,
                "Expected `play` result to be `Ok(..)`, but was {:?}",
                fail
            ),
        };
    }
}
