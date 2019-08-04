//! Components for the transform processor.

#[doc(no_inline)]
pub use self::{
    parent::{HierarchyEvent, Parent, ParentHierarchy},
    transform::Transform,
};

mod parent;
mod transform;
