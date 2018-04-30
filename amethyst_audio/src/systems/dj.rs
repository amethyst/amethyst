use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use amethyst_core::shred::{Resource, Resources};
use amethyst_core::specs::common::Errors;
use amethyst_core::specs::prelude::{Read, System, WriteExpect};

use output::{default_output, Output};
use sink::AudioSink;
use source::{Source, SourceHandle};

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
        if let Some(o) = default_output() {
            res.entry::<AudioSink>()
                .or_insert_with(|| AudioSink::new(&o));
            res.entry::<Output>().or_insert_with(|| o);
        } else {
            error!(
                "Failed finding a default audio output to hook AudioSink to, audio will not work!"
            )
        }
    }
}
