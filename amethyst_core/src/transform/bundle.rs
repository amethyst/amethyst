//! ECS transform bundle

use specs::prelude::{DispatcherBuilder, World};
use specs_hierarchy::HierarchySystem;

use bundle::{ECSBundle, Result};
use transform::*;

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
pub struct TransformBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> TransformBundle<'a> {
    /// Create a new transform bundle
    pub fn new() -> Self {
        Default::default()
    }

    /// Set dependencies for the `TransformSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> ECSBundle<'a, 'b> for TransformBundle<'c> {
    fn build(
        self,
        _: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        Ok(builder
            .with(
                HierarchySystem::<Parent>::new(),
                "parent_hierarchy_system",
                self.dep,
            )
            .with(
                TransformSystem::new(),
                "transform_system",
                &["parent_hierarchy_system"],
            ))
    }
}
