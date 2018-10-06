use amethyst_core::specs::{
    prelude::Component,
    storage::{FlaggedStorage, NullStorage},
};

/// Hidden mesh component
/// Useful for entities, that should not be rendered, but stay loaded in memory.
#[derive(Clone, Debug, Default)]
pub struct Hidden;

impl Component for Hidden {
    //type Storage = NullStorage<Self>;
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}
