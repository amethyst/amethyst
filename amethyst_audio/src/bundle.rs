//! ECS audio bundles

use amethyst_assets::AssetProcessorSystemBundle;
use amethyst_core::ecs::*;
use amethyst_error::Error;

use crate::{output::Output, source::*, systems::*};

/// Audio bundle
///
/// This will only add the audio system and the asset processor for `Source`.
///
/// `DjSystem` must be added separately if you want to use our background music system.
///
/// The generic N type should be the same as the one in `Transform`.
#[derive(Default, Debug)]
pub struct AudioBundle(Output);

impl SystemBundle for AudioBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder
            .add_system(build_audio_system())
            .add_bundle(AssetProcessorSystemBundle::<Source>::default());
        Ok(())
    }
}
