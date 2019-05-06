//! `amethyst` audio ecs components

pub use self::{audio_emitter::AudioEmitter, audio_listener::AudioListener};

use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Entity, Read, WriteStorage},
    math::{Point3, RealField},
};
use amethyst_error::Error;

use serde::{Deserialize, Serialize};

use crate::output::Output;

mod audio_emitter;
mod audio_listener;

/// `PrefabData` for loading audio components
///
/// For `AudioListener`, the currently registered `Output` in the `World` will be used.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct AudioPrefab<N: RealField> {
    emitter: bool,
    /// Left, Right
    listener: Option<(Point3<N>, Point3<N>)>,
}

impl<'a, N: RealField> PrefabData<'a> for AudioPrefab<N> {
    type SystemData = (
        WriteStorage<'a, AudioEmitter>,
        WriteStorage<'a, AudioListener<N>>,
        Option<Read<'a, Output>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        if self.emitter {
            system_data.0.insert(entity, AudioEmitter::default())?;
        }
        if let Some((left_ear, right_ear)) = self.listener {
            system_data.1.insert(
                entity,
                AudioListener {
                    left_ear,
                    right_ear,
                },
            )?;
        }
        Ok(())
    }
}
