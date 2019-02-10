use std::io::Cursor;

use rodio::{Decoder, Sink};

use crate::{output::Output, source::Source, DecoderError};

/// This structure provides a way to programmatically pick and play music.
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
