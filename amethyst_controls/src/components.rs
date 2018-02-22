use specs::{Component, NullStorage};

/// Add this to a camera if you want it to be a fly camera.
/// You need to add the FlyCameraBundle or the required systems for it to work.
#[derive(Default)]
pub struct FlyControlTag;

impl Component for FlyControlTag {
    type Storage = NullStorage<FlyControlTag>;
}
