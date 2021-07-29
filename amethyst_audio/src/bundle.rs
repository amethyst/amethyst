//! ECS audio bundles

use log::warn;
use rodio::OutputStream;

// use amethyst_assets::AssetProcessorSystem;
use amethyst_core::ecs::{DispatcherBuilder, Resources, SystemBundle, World};
use amethyst_error::Error;

use crate::{
    output::{init_output, Output},
    systems::{AudioSystem, SelectedListener},
};

/// Bundle for [`AudioSystem`]; initializes output and loads necessary resources.
///
/// This will initialize audio output by loading an [`OutputStream`], an [`Output`], and an empty
/// [`SelectedListener`] to the world's resources. If no audio output devices are available in the
/// system, it will not load any of these resources.
///
/// [`DjSystem`] must be added separately if you want to use our background music system.
///
/// [`DjSystem`]: struct.DjSystem.html
#[derive(Default, Debug)]
pub struct AudioBundle;

impl SystemBundle for AudioBundle {
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
            resources.get_or_default::<SelectedListener>();
        } else {
            warn!("The default audio device is not available, sound will not work!");
        }

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
