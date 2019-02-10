use std::marker::PhantomData;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    shred::{Resource, Resources},
    specs::{
        common::Errors,
        prelude::{Read, System, WriteExpect},
    },
};

use crate::{
    output::init_output,
    sink::AudioSink,
    source::{Source, SourceHandle},
};

/// Calls a closure if the `AudioSink` is empty.
pub struct DjSystem<F, R> {
    f: F,
    marker: PhantomData<R>,
}

impl<F, R> DjSystem<F, R> {
    /// Creates a new `DjSystem` with the music picker being `f`.
    /// The closure takes a parameter, which needs to be a reference to
    /// a resource type, e.g. `&MusicLibrary`. This resource will be fetched
    /// by the system and passed to the picker.
    pub fn new(f: F) -> Self {
        DjSystem {
            f,
            marker: PhantomData,
        }
    }
}

impl<'a, F, R> System<'a> for DjSystem<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle>,
    R: Resource,
{
    type SystemData = (
        Read<'a, AssetStorage<Source>>,
        Read<'a, Errors>,
        Option<Read<'a, AudioSink>>,
        WriteExpect<'a, R>,
    );

    fn run(&mut self, (storage, errors, sink, mut res): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("dj_system");
        if let Some(ref sink) = sink {
            if sink.empty() {
                if let Some(source) = (&mut self.f)(&mut res).and_then(|h| storage.get(&h)) {
                    errors.execute(|| sink.append(source));
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        init_output(res);
    }
}
