//! Components for the transform processor.

pub use self::{
    parent::{HierarchyEvent, ParentComponent, ParentHierarchy},
    transform::TransformComponent,
};

mod parent;
mod transform;
