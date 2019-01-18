use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::prelude::{Component, Entity, HashMapStorage, NullStorage, WriteStorage};

use serde::{Deserialize, Serialize};

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FlyControlTag;

impl Component for FlyControlTag {
    type Storage = NullStorage<FlyControlTag>;
}

/// To add an arc ball behaviour, add this to a camera which already has the FlyControlTag added.
#[derive(Debug, Clone)]
pub struct ArcBallControlTag {
    /// The target entity which the camera will orbit
    pub target: Entity,
    /// The distance from the target entity that the camera should orbit at.
    pub distance: f32,
}

impl Component for ArcBallControlTag {
    // we can use HashMapStorage here because, according to the specs doc, this storage should be
    // use when the component is used with few entity, I think there will rarely more than one
    // camera
    type Storage = HashMapStorage<ArcBallControlTag>;
}

/// `PrefabData` for loading control tags on an `Entity`
///
/// Will always load a `FlyControlTag`
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ControlTagPrefab {
    /// Place `ArcBallControlTag` on the `Entity`
    pub arc_ball: Option<(usize, f32)>,
}

impl<'a> PrefabData<'a> for ControlTagPrefab {
    type SystemData = (
        WriteStorage<'a, FlyControlTag>,
        WriteStorage<'a, ArcBallControlTag>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), PrefabError> {
        system_data.0.insert(entity, FlyControlTag)?;
        if let Some((index, distance)) = self.arc_ball {
            system_data.1.insert(
                entity,
                ArcBallControlTag {
                    target: entities[index],
                    distance,
                },
            )?;
        }
        Ok(())
    }
}
