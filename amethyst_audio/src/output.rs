//! Provides structures and functions used to get audio outputs.

// We have to use types from this to provide an output iterator type.
use std::{
    convert::TryFrom,
    fmt::{Debug, Formatter, Result as FmtResult},
    io::Cursor,
    ops::{Deref, DerefMut},
};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device,
};
use log::error;
use rodio::{
    Decoder, Devices, OutputDevices, OutputStream, OutputStreamHandle, Sink, Source as RSource,
    StreamError,
};

use crate::{source::Source, DecoderError};

/// Non-Send/Sync type to store the audio device and output stream.
pub struct OutputDevice {
    pub(crate) device: Device,
}

/// Convenience method for opening the default output device.
///
/// Since most modern hardware features audio output, this implementation fails if a device can't
/// be initialized. Use an alternative initialization scheme if running on hardware without an
/// integrated audio chip.
impl Default for OutputDevice {
    fn default() -> Self {
        let output_device = default_output_device().expect("Failed to get default output device.");

        error!("Using: {}", output_device.device.name().unwrap());

        output_device
    }
}

impl OutputDevice {
    /// Returns a new `OutputDevice`.
    pub fn new(device: Device) -> Self {
        OutputDevice { device }
    }

    /// Gets the name of the output
    pub fn name(&self) -> String {
        self.device.name().unwrap_or_else(|e| {
            error!("Failed to determine output device name: {}", e);
            String::from("<unnamed_output_device>")
        })
    }

    /// Returns the rodio device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Returns a new `Output` from this device.
    pub fn output(&self) -> Output {
        Output::try_from(&self.device).expect("Failed to create output device.")
    }
}

impl Debug for OutputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("OutputDevice")
            .field("device", &self.name())
            .finish()
    }
}

/// A speaker(s) through which audio can be played.
///
/// By convention, the default output is stored as a resource in the `World`.
pub struct Output {
    #[allow(unused)]
    pub(crate) stream: OutputStream,
    pub(crate) stream_handle: OutputStreamHandle,
}

impl Deref for Output {
    type Target = OutputStreamHandle;

    fn deref(&self) -> &Self::Target {
        &self.stream_handle
    }
}

impl DerefMut for Output {
    fn deref_mut(&mut self) -> &mut OutputStreamHandle {
        &mut self.stream_handle
    }
}

impl Output {
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
        Sink::try_new(&self.stream_handle)
            .map_err(|_| DecoderError)
            .and_then(|sink| {
                for _ in 0..n {
                    sink.append(
                        Decoder::new(Cursor::new(source.clone()))
                            .map_err(|_| DecoderError)?
                            .amplify(volume),
                    );
                }
                sink.detach();
                Ok(())
            })
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Output").finish()
    }
}

impl TryFrom<&Device> for Output {
    type Error = StreamError;
    fn try_from(device: &Device) -> Result<Self, StreamError> {
        rodio::OutputStream::try_from_device(device).map(|(stream, stream_handle)| Output {
            stream,
            stream_handle,
        })
    }
}

/// An iterator over outputs
#[allow(missing_debug_implementations)]
pub struct OutputIterator {
    devices: OutputDevices<Devices>,
}

impl Iterator for OutputIterator {
    type Item = OutputDevice;

    fn next(&mut self) -> Option<OutputDevice> {
        self.devices.next().map(OutputDevice::new)
    }
}

/// Get the default output, returns none if no outputs are available.
pub fn default_output_device() -> Option<OutputDevice> {
    cpal::default_host()
        .default_output_device()
        .map(OutputDevice::new)
}

/// Get a list of outputs available to the system.
pub fn outputs() -> OutputIterator {
    let devices = cpal::default_host()
        .output_devices()
        .unwrap_or_else(|e| panic!("Error retrieving output devices: `{}`", e));
    OutputIterator { devices }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use {
        crate::{output::OutputDevice, source::Source, DecoderError},
        amethyst_utils::app_root_dir::application_root_dir,
        std::{fs::File, io::Read, vec::Vec},
    };

    #[test]
    #[cfg(all(feature = "wav", target_os = "linux"))]
    fn test_play_wav() {
        test_play("tests/sound_test.wav", true)
    }

    #[test]
    #[cfg(all(feature = "mp3", target_os = "linux"))]
    fn test_play_mp3() {
        test_play("tests/sound_test.mp3", true);
    }

    #[test]
    #[cfg(all(feature = "flac", target_os = "linux"))]
    fn test_play_flac() {
        test_play("tests/sound_test.flac", true);
    }

    #[test]
    #[cfg(all(feature = "vorbis", target_os = "linux"))]
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
        let output_device = OutputDevice::default();
        let output = output_device.output();
        let output = &output;

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
