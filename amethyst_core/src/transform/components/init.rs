use specs::{Component, NullStorage};

/// Initialization flag.
/// Added to entity with a `LocalTransform` component after the first update.
#[derive(Default, Copy, Clone)]
pub struct Init;

impl Component for Init {
    type Storage = NullStorage<Init>;
}
