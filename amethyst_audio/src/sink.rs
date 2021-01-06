use std::io::Cursor;

use rodio::{Decoder, Sink};

use crate::{output::Output, source::Source, DecoderError};

/// This structure provides a way to programmatically pick and play music.
// TODO: This needs a proper debug implementeation. This should probably propigate up to a TODO
// for rodeo, as its missing them as well.
#[allow(missing_debug_implementations)]
pub struct AudioSink {
    sink: Sink,
}

impl AudioSink {
    /// Creates a new `AudioSink` using the given audio output.
    pub fn new(output: &Output) -> AudioSink {
        AudioSink {
            sink: Sink::new(&output.device),
        }
    }

    /// Adds a source to the sink's queue of music to play.
    pub fn append(&self, source: &Source) -> Result<(), DecoderError> {
        self.sink
            .append(Decoder::new(Cursor::new(source.clone())).map_err(|_| DecoderError)?);
        Ok(())
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

    /// Pauses playback, this can be resumed with `AudioSink::play`
    pub fn pause(&self) {
        self.sink.pause()
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
        crate::{output::Output, source::Source, AudioSink},
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

        // Create a Output and AudioSink
        let output = Output::default();
        let sink = AudioSink::new(&output);

        // Call play
        match sink.append(&src) {
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
