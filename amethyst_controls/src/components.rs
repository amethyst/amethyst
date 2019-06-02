use amethyst_assets::PrefabData;
use amethyst_core::{
    ecs::prelude::{Component, Entity, HashMapStorage, NullStorage, WriteStorage},
    Float,
};
use amethyst_error::Error;

use serde::{Deserialize, Serialize};

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FlyControlTagComponent;

impl Component for FlyControlTagComponent {
    type Storage = NullStorage<FlyControlTagComponent>;
}

/// To add an arc ball behaviour, add this to a camera which already has the FlyControlTagComponent added.
#[derive(Debug, Clone)]
pub struct ArcBallControlTagComponent {
    /// The target entity which the camera will orbit
    pub target: Entity,
    /// The distance from the target entity that the camera should orbit at.
    pub distance: Float,
}

impl Component for ArcBallControlTagComponent {
    // we can use HashMapStorage here because, according to the specs doc, this storage should be
    // use when the component is used with few entity, I think there will rarely more than one
    // camera
    type Storage = HashMapStorage<ArcBallControlTagComponent>;
}

/// `PrefabData` for loading control tags on an `Entity`
///
/// Will always load a `FlyControlTagComponent`
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ControlTagPrefab {
    /// Place `ArcBallControlTagComponent` on the `Entity`
    pub arc_ball: Option<(usize, f32)>,
}

impl<'a> PrefabData<'a> for ControlTagPrefab {
    type SystemData = (
        WriteStorage<'a, FlyControlTagComponent>,
        WriteStorage<'a, ArcBallControlTagComponent>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        _: &[Entity],
    ) -> Result<(), Error> {
        system_data.0.insert(entity, FlyControlTagComponent)?;
        if let Some((index, distance)) = self.arc_ball {
            system_data.1.insert(
                entity,
                ArcBallControlTagComponent {
                    target: entities[index],
                    distance: Float::from(distance),
                },
            )?;
        }
        Ok(())
    }
}
