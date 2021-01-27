//! ECS audio bundles

//use amethyst_assets::AssetProcessorSystemBundle;
use amethyst_core::ecs::*;
use amethyst_error::Error;

use crate::{output::OutputWrapper, systems::*};

/// Audio bundle
///
/// This will add an empty SelectedListener, OutputWrapper, add the audio system and the asset processor for `Source`.
///
/// `DjSystem` must be added separately if you want to use our background music system.
#[derive(Default, Debug)]
pub struct AudioBundle;

impl SystemBundle for AudioBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        resources.get_or_default::<OutputWrapper>();
        resources.get_or_default::<SelectedListener>();

        builder.add_system(AudioSystem);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_audio_bundle_should_not_crash_when_executing_iter() {
        let mut resources = Resources::default();
        let mut world = World::default();

        let mut dispatcher = DispatcherBuilder::default()
            .add_bundle(AudioBundle)
            .build(&mut world, &mut resources)
            .unwrap();

        dispatcher.execute(&mut world, &mut resources);
    }
}
