//! ECS transform bundle

use app::ApplicationBuilder;
use ecs::ECSBundle;
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

impl<'a, 'b, 'c, T> ECSBundle<'a, 'b, T> for TransformBundle<'c> {
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        Ok(
            builder
                .register::<Init>()
                .register::<Child>()
                .register::<LocalTransform>()
                .register::<Transform>()
                .with(TransformSystem::new(), "transform_system", self.dep),
        )
    }
}
