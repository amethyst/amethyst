use std::io::Cursor;

use rodio::{Decoder, Sink};

use audio::{DecoderError, Source};
use audio::output::Output;

/// This structure provides a way to programmatically pick and play music.
pub struct Dj {
    sink: Sink,
    pub(crate) picker: Option<Box<FnMut(&mut Dj) -> bool + Send + Sync>>
}

impl Dj {
    /// Creates a new Dj using the given audio output.
    pub fn new(output: &Output) -> Dj {
        Dj {
            sink: Sink::new(&output.endpoint),
            picker: None,
        }
    }

    /// A Dj's picker will be called by the DjSystem whenever the Dj runs out of music to play.
    ///
    /// Only the Dj added to the world's resources with resource ID 0 will have their picker called.
    ///
    /// During callback the picker is separated from the Dj in order to avoid multiple aliasing.
    /// After the callback is complete, if the picker returned true it will be reattached.
    pub fn set_picker(&mut self, picker: Box<FnMut(&mut Dj) -> bool + Send + Sync>) {
        self.picker = Some(picker);
    }

    /// Clears the previously set picker.
    pub fn clear_picker(&mut self) {
        self.picker = None;
    }

    /// Adds a source to the Dj's queue of music to play.
    pub fn append(&self, source: &Source) -> Result<(), DecoderError> {
        self.sink.append(Decoder::new(Cursor::new(source.clone())).map_err(|_| DecoderError)?);
        Ok(())
    }

    /// Returns true if the Dj has no more music to play.
    pub fn empty(&self) -> bool {
        self.sink.empty()
    }

    /// Retrieves the volume of the Dj, between 0.0 and 1.0;
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    /// Sets the volume of the Dj.
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
    }

    /// Resumes playback of a paused Dj.  Has no effect if this Dj was never paused.
    pub fn play(&self) {
        self.sink.play();
    }

    /// Pauses playback, this can be resumed with `Dj::play`
    pub fn pause(&self) {
        self.sink.pause()
    }

    /// Returns true if the Dj is currently paused.
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Empties the Dj's queue of all music.
    pub fn stop(&self) {
        self.sink.stop();
    }
}
