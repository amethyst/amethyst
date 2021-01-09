use std::{
    io::Cursor,
    sync::{atomic::AtomicBool, Arc},
};

use rodio::{Decoder, SpatialSink};
use smallvec::SmallVec;

use crate::{source::Source, DecoderError};

/// An audio source, add this component to anything that emits sound.
/// TODO: This should get a proper Debug impl parsing the sinks and sound queue
#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct AudioEmitter {
    pub(crate) sinks: SmallVec<[(SpatialSink, Arc<AtomicBool>); 4]>,
    pub(crate) sound_queue: SmallVec<[Decoder<Cursor<Source>>; 4]>,
    pub(crate) picker: Option<Box<dyn FnMut(&mut AudioEmitter) -> bool + Send + Sync>>,
}

impl AudioEmitter {
    /// Creates a new AudioEmitter component initialized to the given positions.
    /// These positions will stay synced with Transform if the Transform component is available
    /// on this entity.
    pub fn new() -> AudioEmitter {
        Default::default()
    }

    /// Plays an audio source from this emitter.
    pub fn play(&mut self, source: &Source) -> Result<(), DecoderError> {
        self.sound_queue
            .push(Decoder::new(Cursor::new(source.clone())).map_err(|_| DecoderError)?);
        Ok(())
    }

    /// An emitter's picker will be called by the AudioSystem whenever the emitter runs out of
    /// sounds to play.
    ///
    /// During callback the picker is separated from the emitter in order to avoid multiple
    /// aliasing.
    /// After the callback is complete, if the picker returned true then the
    /// picker that just finished will be reattached.
    pub fn set_picker(&mut self, picker: Box<dyn FnMut(&mut AudioEmitter) -> bool + Send + Sync>) {
        self.picker = Some(picker);
    }

    /// Clears the previously set picker.
    pub fn clear_picker(&mut self) {
        self.picker = None;
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, vec::Vec};

    use amethyst_utils::app_root_dir::application_root_dir;

    use crate::{AudioEmitter, Source};

    // test_play tests the AudioEmitter's play function
    fn test_play(file_name: &str, should_pass: bool) {
        // Get the full file path
        let app_root = application_root_dir().unwrap();
        let audio_path = app_root.join(file_name);

        // Convert the file contents into a byte vec
        let mut f = File::open(audio_path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();

        // Create a Source and AudioEmitter from those bytes
        let src = Source { bytes: buffer };
        let mut emitter = AudioEmitter::default();

        // Call play
        match emitter.play(&src) {
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
    fn test_play_wav() {
        test_play("tests/sound_test.wav", true);
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

    // test_picker tests the set and clear picker functions
    #[test]
    fn test_picker() {
        // Create the input variables
        let mut emitter_main = AudioEmitter::default();
        let box_picker: Box<dyn FnMut(&mut AudioEmitter) -> bool + Send + Sync> =
            Box::new(use_audio_emitter);

        // Test set_picker and assert that it is not empty
        emitter_main.set_picker(box_picker);
        assert!(emitter_main.picker.is_some());

        // Test clear_picker and assert it is empty
        emitter_main.clear_picker();
        assert!(emitter_main.picker.is_none());
    }

    // use_audio_emitter is a fake test function to play an AudioEmitter
    fn use_audio_emitter(_emitter: &mut AudioEmitter) -> bool {
        true
    }
}
