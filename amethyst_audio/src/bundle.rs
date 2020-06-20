//! ECS audio bundles

use amethyst_assets::build_asset_processor_system;
use amethyst_core::{
    dispatcher::{DispatcherBuilder, Stage, SystemBundle},
    ecs::prelude::*,
};
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
    fn build(
        self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        builder.add_system(Stage::Begin, build_audio_system);
        builder.add_system(Stage::Begin, build_asset_processor_system::<Source>);
        Ok(())
    }
}
