use amethyst_core::specs::prelude::{Component, Entity, HashMapStorage, NullStorage};

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyControlBundle or the required systems for it to work.
#[derive(Default)]
pub struct FlyControlTag;

impl Component for FlyControlTag {
    type Storage = NullStorage<FlyControlTag>;
}

/// To add an arc ball behaviour, add this to a camera which already has the FlyControlTag added.
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
