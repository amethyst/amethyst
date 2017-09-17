//! ECS transform bundle

use ecs::{ECSBundle, World, DispatcherBuilder};
use ecs::transform::*;
use error::Result;

/// Transform bundle
///
/// Will register transform components, and the TransformSystem.
/// TransformSystem will be registered with name "transform_system".
///
pub struct TransformBundle<'a> {
    dep: &'a [&'a str],
}

impl<'a> TransformBundle<'a> {
    /// Create a new transform bundle
    pub fn new() -> Self {
        Self { dep: &[] }
    }

    /// Set dependencies for the TransformSystem
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, A> ECSBundle<'a, 'b, A> for TransformBundle<'c> {
    fn build(
        &self,
        _: A,
        world: &mut World,
        mut dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        world.register::<Init>();
        world.register::<Child>();
        world.register::<LocalTransform>();
        world.register::<Transform>();

        dispatcher = dispatcher.add(TransformSystem::new(), "transform_system", self.dep);

        Ok(dispatcher)
    }
}
