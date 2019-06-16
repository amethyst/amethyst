use crate::ecs::prelude::{Component, DenseVecStorage, Entity, FlaggedStorage};

pub use specs_hierarchy::HierarchyEvent;
use specs_hierarchy::{Hierarchy, Parent as HParent};

/// An alias to tie `specs-hierarchy` `Hierarchy` structure to our `Parent` component.
pub type ParentHierarchy = Hierarchy<Parent>;

/// Component for defining a parent entity.
///
/// The entity with this component *has* a parent, rather than *is* a parent.
///
/// If the parent enitity has a transform, then all of that transform 
/// (scale, then rotation, then translation) will be applied the object
/// before any of the child's tranform is applied. For example, if a 
/// parent rotates 45 degrees then child translations will be as if 
/// the axes are rotated 45 degrees.
///
/// If the parent entity has is own parent (and so on) then the transforms 
/// will all be applied in order from the oldest ancestor to the child.

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, new)]
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
