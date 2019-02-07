//! ECS audio bundles

use amethyst_assets::Processor;
use amethyst_core::{bundle::SystemBundle, specs::prelude::DispatcherBuilder};
use amethyst_error::Error;

use crate::{source::*, systems::AudioSystem};

/// Audio bundle
///
/// This will only add the audio system and the asset processor for `Source`.
///
/// `DjSystem` must be added separately if you want to use our background music system.
///
pub struct AudioBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for AudioBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(AudioSystem::new(), "audio_system", &[]);
        builder.add(Processor::<Source>::new(), "source_processor", &[]);
        Ok(())
    }
}
