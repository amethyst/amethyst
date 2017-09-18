//! ECS audio bundles

use app::ApplicationBuilder;
use audio::Dj;
use audio::output::{default_output, Output};
use ecs::ECSBundle;
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

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b, T> for DjBundle<'c> {
    fn build(
        &self,
        mut builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        // Remove option here when specs get support for optional fetch in
        // released version
        if !builder.world.res.has_value(
            ResourceId::new::<Option<Output>>(),
        )
        {
            builder = builder.with_resource(default_output());
        }

        let dj = match *builder.world.read_resource::<Option<Output>>() {
            Some(ref audio_output) => Some(Dj::new(&audio_output)),

            None => None,
        };

        if let Some(dj) = dj {
            builder = builder.with_resource(dj).with(
                DjSystem,
                "dj_system",
                self.dep,
            );
        }

        Ok(builder)
    }
}
