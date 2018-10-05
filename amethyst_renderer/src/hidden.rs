use amethyst_core::specs::{
    prelude::Component,
    storage::{
        FlaggedStorage,
        NullStorage,
    },
};

/// Hidden mesh component
/// Useful for Ui-Elements that should stay loaded, but not visible.
#[derive(Clone, Debug, Default)]
pub struct Hidden;

impl Component for Hidden{
    //type Storage = NullStorage<Self>;
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}
