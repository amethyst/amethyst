//! ECS audio bundles

use std::marker::PhantomData;

use rodio::default_output_device;

use amethyst_assets::Processor;
use amethyst_core::{
    bundle::{Result, SystemBundle},
    specs::prelude::DispatcherBuilder,
};

use crate::{source::*, systems::DjSystem};

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
pub struct AudioBundle<'a, F, R> {
    dep: &'a [&'a str],
    marker: PhantomData<R>,
    picker: Option<F>,
}

impl<'a, F, R> AudioBundle<'a, F, R> {
    /// Create a new audio bundle
    pub fn new() -> Self {
        AudioBundle {
            dep: &[],
            marker: PhantomData,
            picker: None,
        }
    }

    /// Initialize the DJ system
    pub fn with_dj_system(mut self, picker: F) -> Self {
        self.picker = Some(picker);
        self
    }

    /// Set dependencies for the `DjSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, F, R> SystemBundle<'a, 'b> for AudioBundle<'c, F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle> + Send + 'static,
    R: Send + Sync + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(Processor::<Source>::new(), "source_processor", &[]);
        if default_output_device().is_some() && self.picker.is_some() {
            builder.add(DjSystem::new(self.picker.unwrap()), "dj_system", self.dep);
        }
        Ok(())
    }
}
