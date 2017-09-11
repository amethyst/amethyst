
use specs::{Component, DenseVecStorage, Entity, FlaggedStorage};

/// Component for defining a parent entity.
pub struct Parent {
    /// The parent entity
    pub entity: Entity,
}

impl Component for Parent {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
