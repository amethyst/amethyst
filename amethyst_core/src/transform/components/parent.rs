use crate::ecs::prelude::{Component, DenseVecStorage, Entity, FlaggedStorage};

pub use specs_hierarchy::HierarchyEvent;
use specs_hierarchy::{Hierarchy, Parent as HParent};

/// An alias to tie `specs-hierarchy` `Hierarchy` structure to our `ParentComponent` component.
pub type ParentHierarchy = Hierarchy<ParentComponent>;

/// Component for defining a parent entity.
///
/// The entity with this component *has* a parent, rather than *is* a parent.
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, new)]
pub struct ParentComponent {
    /// The parent entity
    pub entity: Entity,
}

impl Component for ParentComponent {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl HParent for ParentComponent {
    fn parent_entity(&self) -> Entity {
        self.entity
    }
}
