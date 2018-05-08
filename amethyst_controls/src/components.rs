use amethyst_core::specs::prelude::{Component, Entity};
use amethyst_core::specs::storage::{NullStorage, HashMapStorage};

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Default)]
pub struct FlyControlTag;

impl Component for FlyControlTag {
    type Storage = NullStorage<FlyControlTag>;
}

pub struct ArcBallCameraTag {
    pub target: Entity,
}

impl Component for ArcBallCameraTag {
    // we can use HashMapStorage here because, according to the specs doc, this storage should be
    // use when the component is use with few entity, I think there will rarely more than one
    // camera
    type Storage = HashMapStorage<ArcBallCameraTag>;
}
