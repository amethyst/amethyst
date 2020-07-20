use amethyst_core::ecs::prelude::*;

use derive_new::new;
use serde::{Deserialize, Serialize};

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FlyControl;

/// To add an arc ball behaviour, add this to a camera which already has the FlyControlTag added.
#[derive(Debug, Clone, new)]
pub struct ArcBallControl {
    /// The target entity which the camera will orbit
    pub target: Entity,
    /// The distance from the target entity that the camera should orbit at.
    pub distance: f32,
}

// `PrefabData` for loading control tags on an `Entity`
//
// Will always load a `FlyControlTag`
// #[derive(Debug, Default, Clone, Deserialize, Serialize)]
// pub struct ControlTagPrefab {
//     /// Place `ArcBallControlTag` on the `Entity`
//     pub arc_ball: Option<(usize, f32)>,
// }
//
// impl<'a> PrefabData<'a> for ControlTagPrefab {
//     type SystemData = (
//         WriteStorage<'a, FlyControl>,
//         WriteStorage<'a, ArcBallControl>,
//     );
//     type Result = ();
//
//     fn add_to_entity(
//         &self,
//         entity: Entity,
//         system_data: &mut Self::SystemData,
//         entities: &[Entity],
//         _: &[Entity],
//     ) -> Result<(), Error> {
//         system_data.0.insert(entity, FlyControl)?;
//         if let Some((index, distance)) = self.arc_ball {
//             system_data.1.insert(
//                 entity,
//                 ArcBallControl {
//                     target: entities[index],
//                     distance,
//                 },
//             )?;
//         }
//         Ok(())
//     }
// }
