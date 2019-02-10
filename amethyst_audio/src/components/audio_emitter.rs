use std::{
    io::Cursor,
    sync::{atomic::AtomicBool, Arc},
};

use rodio::{Decoder, SpatialSink};
use smallvec::SmallVec;

use amethyst_core::specs::{prelude::Component, storage::BTreeStorage};

use crate::{source::Source, DecoderError};

/// An audio source, add this component to anything that emits sound.
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

impl Component for AudioEmitter {
    type Storage = BTreeStorage<Self>;
}
