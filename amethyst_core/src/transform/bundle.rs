//! ECS transform bundle

use amethyst_error::Error;
use specs_hierarchy::HierarchySystem;

use crate::{
    bundle::SystemBundle,
    ecs::prelude::{DispatcherBuilder, World},
    transform::*,
    SystemDesc,
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
#[derive(Debug, Default)]
pub struct TransformBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> TransformBundle<'a> {
    /// Create a new transform bundle
    pub fn new() -> Self {
        TransformBundle {
            dep: Default::default(),
        }
    }

    /// Set dependencies for the `TransformSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c> SystemBundle<'a, 'b> for TransformBundle<'c> {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            HierarchySystem::<Parent>::new(world),
            "parent_hierarchy_system",
            self.dep,
        );
        builder.add(
            TransformSystemDesc::default().build(world),
            "transform_system",
            &["parent_hierarchy_system"],
        );
        Ok(())
    }
}
