use log::error;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use amethyst_assets::AssetStorage;
use amethyst_core::ecs::*;
use amethyst_error::Error;

use crate::{
    output::init_output,
    sink::AudioSink,
    source::{Source, SourceHandle},
};

/// Dj system bundle which is the default way to construct dj system as it initializes any required resources.
#[derive(Debug)]
pub struct DjSystemBundle<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    f: Option<F>,
    _phantom: std::marker::PhantomData<R>,
}

impl<F, R> DjSystemBundle<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    /// Creates a new [DjSystemBundle] where [f] is a function which produces music [SourceHandle].
    pub fn new(f: F) -> Self {
        Self {
            f: Some(f),
            _phantom: Default::default(),
        }
    }
}

impl<F, R> SystemBundle for DjSystemBundle<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        init_output(resources);
        builder.add_system(build_dj_system(
            self.f
                .take()
                .expect("DJ system function not provided or bundle loaded multiple times"),
        ));
        Ok(())
    }
}

/// Calls a closure if the `AudioSink` is empty.
pub fn build_dj_system<F, R>(mut f: F) -> impl Runnable
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    SystemBuilder::new("DjSystem")
        .read_resource::<AssetStorage<Source>>()
        .read_resource::<Option<AudioSink>>()
        .write_resource::<R>()
        .build(move |_commands, _world, (storage, sink, res), _queries| {
            #[cfg(feature = "profiler")]
            profile_scope!("dj_system");

            if let Some(sink) = &**sink {
                if sink.empty() {
                    if let Some(source) = f(res).and_then(|h| storage.get(&h)) {
                        if let Err(e) = sink.append(source) {
                            error!("DJ Cannot append source to sink. {}", e);
                        }
                    }
                }
            }
        })
}
