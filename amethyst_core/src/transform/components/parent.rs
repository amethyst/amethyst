use specs::prelude::{Component, DenseVecStorage, Entity, FlaggedStorage};
pub use specs_hierarchy::HierarchyEvent;
use specs_hierarchy::{Hierarchy, Parent as HParent};
pub type ParentHierarchy = Hierarchy<Parent>;

/// Component for defining a parent entity.
///
/// The entity with this component *has* a parent, rather than *is* a parent.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Parent {
    /// The parent entity
    pub entity: Entity,
}

impl Component for Parent {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl HParent for Parent {
    fn parent_entity(&self) -> Entity {
        self.entity
    }
}
