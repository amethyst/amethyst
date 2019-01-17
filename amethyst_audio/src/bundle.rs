//! ECS audio bundles

use amethyst_assets::Processor;
use amethyst_core::{
    bundle::{Result, SystemBundle},
    specs::prelude::DispatcherBuilder,
};

use crate::{
    systems::AudioSystem,
    output::Output,
    source::*
};

/// Audio bundle
///
/// This will only add the asset processor for `Source`.
///
/// `DjSystem` must be added separately if you want to use our background music system.
///
#[derive(Default)]
pub struct AudioBundle(Output);

impl<'a, 'b> SystemBundle<'a, 'b> for AudioBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(AudioSystem::new(self.0), "audio_system", &[]);
        builder.add(Processor::<Source>::new(), "source_processor", &[]);
        Ok(())
    }
}
