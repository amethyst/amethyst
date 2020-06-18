use log::error;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::AssetStorage;
use amethyst_core::{
    ecs::prelude::*,
};

use crate::{
    output::init_output,
    sink::AudioSink,
    source::{Source, SourceHandle},
};

/// Calls a closure if the `AudioSink` is empty.
pub fn build_dj_system<F, R>(f: F) -> Box<dyn FnOnce (&mut World, &mut Resources) -> Box<dyn Schedulable>>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    Box::new(|world: &mut World, res: &mut Resources| {
        init_output(res);
        SystemBuilder::<()>::new("DjSystem")
            .read_resource::<AssetStorage<Source>>()
            .read_resource::<Option<AudioSink>>()
            .write_resource::<R>()
            .build(move |_commands, _world, (storage, sink, mut res), _queries| {
                #[cfg(feature = "profiler")]
                profile_scope!("dj_system");

                if let Some(sink) = **sink {
                    if sink.empty() {
                        if let Some(source) = (&mut f)(&mut res).and_then(|h| storage.get(&h)) {
                            if let Err(e) = sink.append(source) {
                                error!("DJ Cannot append source to sink. {}", e);
                            }
                        }
                    }
                }
            })
    })
}
