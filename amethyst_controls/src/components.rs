use amethyst_core::specs::prelude::{Component, Entity};
use amethyst_core::specs::storage::{NullStorage, HashMapStorage};

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Default)]
pub struct FlyControlTag;

impl Component for FlyControlTag {
    type Storage = NullStorage<FlyControlTag>;
}

/// Add this to a camera to which you have already add the FlyControlTag to add a arc ball
/// behaviour to the camera.
/// Please, note that this component requires the ArcBallControlSystem to work.
#[derive(Debug, Clone)]
pub struct ArcBallControlTag {
    pub target: Entity,
    pub distance: f32,
}

impl Component for ArcBallControlTag {
    // we can use HashMapStorage here because, according to the specs doc, this storage should be
    // use when the component is used with few entity, I think there will rarely more than one
    // camera
    type Storage = HashMapStorage<ArcBallControlTag>;
}
