//! ECS transform bundle

use specs::prelude::{DispatcherBuilder, System, World};
use specs_hierarchy::{Hierarchy, HierarchySystem};

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
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.register::<Transform>();
        HierarchySystem::<Parent>::setup(&mut world.res);

        let mut locals = world.write::<Transform>();
        let mut hierarchy = world.write_resource::<Hierarchy<Parent>>();

        Ok(builder
            .with(
                HierarchySystem::<Parent>::new(),
                "parent_hierarchy_system",
                self.dep,
            )
            .with(
                TransformSystem::new(
                    locals.track_inserted(),
                    locals.track_modified(),
                    hierarchy.track(),
                ),
                "transform_system",
                &["parent_hierarchy_system"],
            ))
    }
}
