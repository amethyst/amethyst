use amethyst_core::ecs::*;
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
