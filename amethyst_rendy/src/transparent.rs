use amethyst_core::ecs::{prelude::Component, storage::NullStorage};

/// Transparent mesh component
#[derive(Clone, Debug, Default)]
pub struct Transparent;

impl Component for Transparent {
    type Storage = NullStorage<Self>;
}
