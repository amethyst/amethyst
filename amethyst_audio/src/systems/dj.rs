use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use amethyst_core::ecs::{
    DispatcherBuilder, ParallelRunnable, Resources, System, SystemBuilder, SystemBundle, World,
};
use amethyst_error::Error;

use log::{error, warn};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    output::{init_output, Output, OutputStream},
    source::{Source, SourceHandle},
};

/// Dj system bundle which is the default way to construct dj system as it initializes any required resources.
#[derive(Debug)]
pub struct DjSystemBundle<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    f: F,
    _marker: PhantomData<R>,
}

impl<F, R> DjSystemBundle<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    /// Creates a new [`DjSystemBundle`] where [f] is a function which produces music [`SourceHandle`].
    pub fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

impl<F, R> SystemBundle for DjSystemBundle<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static + Copy,
    R: Send + Sync + 'static,
{
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        // Try to initialize output using the system's default audio device.
        if let Ok((stream, output)) = init_output() {
            resources.get_or_insert::<OutputStream>(stream);
            resources.get_or_insert::<Output>(output);
        } else {
            warn!("The default audio device is not available, sound will not work!");
        }

        builder.add_system(DjSystem {
            f: self.f,
            _phantom: PhantomData,
        });

        Ok(())
    }
}

/// Calls a closure if the `AudioSink` is empty.
#[derive(Debug, Clone)]
pub struct DjSystem<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync,
    R: Send + Sync,
{
    f: F,
    _phantom: std::marker::PhantomData<R>,
}

impl<F, R> System for DjSystem<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    fn build(mut self) -> Box<dyn ParallelRunnable + 'static> {
        Box::new(
            SystemBuilder::new("DjSystem")
                .read_resource::<AssetStorage<Source>>()
                .read_resource::<Output>()
                .write_resource::<R>()
                .build(move |_commands, _world, (storage, output, res), _queries| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("dj_system");

                    let sink = output.try_spawn_sink().unwrap();

                    if sink.empty() {
                        if let Some(source) = (self.f)(res).and_then(|h| storage.get(&h)) {
                            if let Err(e) = sink.append(source, 1.0) {
                                error!("DJ cannot append source to sink. {}", e);
                            }
                        }
                    }
                }),
        )
    }
}
