use std::marker::PhantomData;

use derive_new::new;
use log::error;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    ecs::prelude::{Read, System, SystemData, World, WriteExpect},
    shred::Resource,
    SystemDesc,
};

use crate::{
    output::init_output,
    sink::AudioSink,
    source::{Source, SourceHandle},
};

/// Creates a new `DjSystem` with the music picker being `f`.
///
/// The closure takes a parameter, which needs to be a reference to a resource type,
/// e.g. `&MusicLibrary`. This resource will be fetched by the system and passed to the picker.
#[derive(Debug, new)]
pub struct DjSystemDesc<F, R> {
    f: F,
    marker: PhantomData<R>,
}

impl<'a, 'b, F, R> SystemDesc<'a, 'b, DjSystem<F, R>> for DjSystemDesc<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle>,
    R: Resource,
{
    fn build(self, world: &mut World) -> DjSystem<F, R> {
        <DjSystem<F, R> as System<'_>>::SystemData::setup(world);

        init_output(world);

        DjSystem::new(self.f)
    }
}

/// Calls a closure if the `AudioSink` is empty.
#[derive(Debug, new)]
pub struct DjSystem<F, R> {
    f: F,
    marker: PhantomData<R>,
}

impl<'a, F, R> System<'a> for DjSystem<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle>,
    R: Resource,
{
    type SystemData = (
        Read<'a, AssetStorage<Source>>,
        Option<Read<'a, AudioSink>>,
        WriteExpect<'a, R>,
    );

    fn run(&mut self, (storage, sink, mut res): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("dj_system");

        if let Some(ref sink) = sink {
            if sink.empty() {
                if let Some(source) = (&mut self.f)(&mut res).and_then(|h| storage.get(&h)) {
                    if let Err(e) = sink.append(source) {
                        error!("DJ Cannot append source to sink. {}", e);
                    }
                }
            }
        }
    }
}
