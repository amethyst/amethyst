//! `amethyst` audio ecs components

pub use self::{audio_emitter::AudioEmitter, audio_listener::AudioListener};

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::{
    nalgebra::Point3,
    specs::prelude::{Entity, Read, WriteStorage},
};

use serde::{Deserialize, Serialize};

use crate::output::Output;

mod audio_emitter;
mod audio_listener;

/// `PrefabData` for loading audio components
///
/// For `AudioListener`, the currently registered `Output` in the `World` will be used.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct AudioPrefab {
    emitter: bool,
    /// Left, Right
    listener: Option<(Point3<f32>, Point3<f32>)>,
}

impl<'a> PrefabData<'a> for AudioPrefab {
    type SystemData = (
        WriteStorage<'a, AudioEmitter>,
        WriteStorage<'a, AudioListener>,
        Option<Read<'a, Output>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        if self.emitter {
            system_data.0.insert(entity, AudioEmitter::default())?;
        }
        if let (Some((left_ear, right_ear)), Some(output)) = (self.listener, &system_data.2) {
            system_data.1.insert(
                entity,
                AudioListener {
                    output: (*output).clone(),
                    left_ear,
                    right_ear,
                },
            )?;
        }
        Ok(())
    }
}
