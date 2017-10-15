//! ECS audio bundles

use core::bundle::{ECSBundle, Result};

use assets::{AssetStorage, Processor};
use audio::{AudioSink, Source, SourceHandle};
use audio::output::{default_output, Output};
use ecs::{DispatcherBuilder, World};
use ecs::audio::DjSystem;
use shred::ResourceId;

/// Audio bundle
///
/// Will only register the `AudioSink` and the `DjSystem` if an audio output is found.
/// `DjSystem` will be registered with name "dj_system".
///
/// This will also add the asset processor for `Source`.
///
/// ## Errors
///
/// No errors returned by this bundle
///
/// ## Panics
///
/// Panics during `DjSystem` registration if the bundle is applied twice.
///
pub struct AudioBundle<'a, F> {
    dep: &'a [&'a str],
    picker: F,
}

impl<'a, F> AudioBundle<'a, F> {
    /// Create a new DJ bundle
    pub fn new(picker: F) -> Self {
        AudioBundle { dep: &[], picker }
    }

    /// Set dependencies for the `DjSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, F> ECSBundle<'a, 'b> for AudioBundle<'c, F>
where
    F: FnMut() -> Option<SourceHandle> + Send + 'static,
{
    fn build(
        self,
        world: &mut World,
        mut builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        // Remove option here when specs get support for optional fetch in
        // released version
        if !world.res.has_value(ResourceId::new::<Option<Output>>()) {
            world.add_resource(default_output());
        }

        let sink = world
            .read_resource::<Option<Output>>()
            .as_ref()
            .map(|audio_output| AudioSink::new(audio_output));

        world.add_resource(AssetStorage::<Source>::new());

        if let Some(sink) = sink {
            world.add_resource(sink);
            builder = builder
                .add(Processor::<Source>::new(), "source_processor", &[])
                .add(DjSystem::new(self.picker), "dj_system", self.dep);
        }

        Ok(builder)
    }
}
