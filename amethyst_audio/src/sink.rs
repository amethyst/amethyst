use std::io::Cursor;

use rodio::{Decoder, OutputStreamHandle, PlayError, Sink as RodioSink, Source as RodioSource};

use crate::{source::Source, DecoderError};

/// This structure provides a way to programmatically pick and play music.
///
/// Please note that unless you `detach()` the sink explicitly, the audio playback stops
/// immediately as the sink gets dropped (goes out of scope, for example).
// TODO: This needs a proper debug implementation. This should probably propagate up to a TODO
// for rodio, as its missing them as well.
#[allow(missing_debug_implementations)]
pub struct Sink {
    sink: RodioSink,
}

impl Sink {
    /// Creates a new `Sink` using the given output stream handle.
    ///
    /// # Errors
    ///
    /// The result is a `PlayError::NoDevice` if there is no output device associated
    /// with the output stream handle provided to create the sink.
    pub fn try_new(stream_handle: &OutputStreamHandle) -> Result<Self, PlayError> {
        RodioSink::try_new(stream_handle).map(|sink| Sink { sink })
    }

    /// Adds a source to the sink's queue of music to play.
    ///
    /// # Errors
    ///
    /// The result is an Error if decoding the audio source fails.
    pub fn append(&self, source: &Source, volume: f32) -> Result<(), DecoderError> {
        self.sink
            .append(Decoder::new(Cursor::new(source.clone()))?.amplify(volume));
        Ok(())
    }

    /// Drops the sink without stopping currently playing sounds.
    ///
    /// When a sink goes out of scope, for example, the audio output stops
    /// immediately. If the sink is detached before it gets dropped, however,
    /// the output continues until all the sounds appended to the sink are played.
    pub fn detach(self) {
        self.sink.detach();
    }

    /// Returns true if the sink has no more music to play.
    pub fn empty(&self) -> bool {
        self.sink.empty()
    }

    /// Retrieves the volume of the sink, between 0.0 and 1.0;
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    /// Sets the volume of the sink.
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
    }

    /// Resumes playback of a paused sink. Has no effect if this sink was never paused.
    pub fn play(&self) {
        self.sink.play();
    }

    /// Pauses playback, this can be resumed with `Sink::play`
    pub fn pause(&self) {
        self.sink.pause();
    }

    /// Returns true if the sink is currently paused.
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Empties the sink's queue of all music.
    pub fn stop(&self) {
        self.sink.stop();
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use {
        crate::{output::init_output, sink::Sink, source::Source},
        amethyst_utils::app_root_dir::application_root_dir,
        std::{fs::File, io::Read, vec::Vec},
    };

    // test_append tests the AudioSink's append function
    #[cfg(target_os = "linux")]
    fn test_append(file_name: &str, should_pass: bool) {
        // Get the full file path
        let app_root = application_root_dir().unwrap();
        let audio_path = app_root.join(file_name);

        // Convert the file contents into a byte vec
        let mut f = File::open(audio_path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();

        // Create a Source from those bytes
        let src = Source { bytes: buffer };

        // Create an Output and a Sink
        let (_stream, output) = init_output().unwrap();
        let sink = Sink::try_new(&output.stream_handle).unwrap();

        // Call play
        match sink.append(&src, 1.0) {
            Ok(_pass) => {
                assert!(
                    should_pass,
                    "Expected `append` result to be Err(..), but was Ok(..)"
                )
            }
            Err(fail) => {
                assert!(
                    !should_pass,
                    "Expected `append` result to be `Ok(..)`, but was {:?}",
                    fail
                )
            }
        };
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_append_wav() {
        test_append("tests/sound_test.wav", true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_append_mp3() {
        test_append("tests/sound_test.mp3", true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_append_flac() {
        test_append("tests/sound_test.flac", true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_play_ogg() {
        test_append("tests/sound_test.ogg", true);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_append_fake() {
        test_append("tests/sound_test.fake", false);
    }
}
