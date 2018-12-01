//! ECS transform bundle

use specs_hierarchy::HierarchySystem;

use crate::{
    bundle::{Result, SystemBundle},
    transform::*,
    SimpleDispatcherBuilder,
};

/// Transform bundle
///
/// Will register transform components, and the `TransformSystem`.
/// `TransformSystem` will be registered with name "transform_system".
///
/// ## Errors
///
/// No errors will be returned by this bundle.
///
/// ## Panics
///
/// Panics in `TransformSystem` registration if the bundle is applied twice in the same dispatcher.
///
#[derive(Default)]
pub struct TransformBundle<'a: 'b, 'b> {
    dep: &'b [&'a str],
}

impl<'a, 'b> TransformBundle<'a, 'b> {
    /// Create a new transform bundle
    pub fn new() -> Self {
        Default::default()
    }

    /// Set dependencies for the `TransformSystem`
    pub fn with_dep(mut self, dep: &'b [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, 'd, D> SystemBundle<'a, 'b, 'c, D> for TransformBundle<'c, 'd>
where
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    fn build(self, builder: &mut D) -> Result<()> {
        builder.add(
            HierarchySystem::<Parent>::new(),
            "parent_hierarchy_system",
            self.dep,
        );
        builder.add(
            TransformSystem::new(),
            "transform_system",
            &["parent_hierarchy_system"],
        );
        Ok(())
    }
}
