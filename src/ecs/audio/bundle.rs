//! ECS audio bundles

use audio::Dj;
use audio::output::{default_output, Output};
use ecs::{ECSBundle, World, DispatcherBuilder};
use ecs::audio::DjSystem;
use error::Result;
use shred::ResourceId;

/// DJ bundle
///
/// Will only register the DjSystem if it can either get the default audio output,
/// or fetch it from the world. DjSystem will be registered with name "dj_system".
pub struct DjBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> DjBundle<'a> {
    /// Create a new DJ bundle
    pub fn new() -> Self {
        Self { dep: &[] }
    }

    /// Set dependencies for the DjSystem
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, A> ECSBundle<'a, 'b, A> for DjBundle<'c> {
    fn build(
        &self,
        _: A,
        world: &mut World,
        mut dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {

        // Remove option here when specs get support for optional fetch in
        // released version
        if !world.res.has_value(ResourceId::new::<Option<Output>>()) {
            world.add_resource(default_output());
        }

        let dj = match *world.read_resource::<Option<Output>>() {
            Some(ref audio_output) => Some(Dj::new(&audio_output)),

            None => None,
        };

        if let Some(dj) = dj {
            world.add_resource(dj);
            dispatcher = dispatcher.add(DjSystem, "dj_system", self.dep);
        }

        Ok(dispatcher)
    }
}
