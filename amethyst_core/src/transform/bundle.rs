//! ECS transform bundle

use std::marker::PhantomData;

use amethyst_error::Error;
use specs_hierarchy::HierarchySystem;

use crate::{bundle::SystemBundle, ecs::prelude::DispatcherBuilder, math::RealField, transform::*};

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
pub struct TransformBundle<'a, N> {
    dep: &'a [&'a str],
    _phantom: PhantomData<N>,
}

impl<'a, N> TransformBundle<'a, N> {
    /// Create a new transform bundle
    pub fn new() -> Self {
        TransformBundle {
            dep: Default::default(),
            _phantom: PhantomData,
        }
    }

    /// Set dependencies for the `TransformSystem`
    pub fn with_dep(mut self, dep: &'a [&'a str]) -> Self {
        self.dep = dep;
        self
    }
}

impl<'a, 'b, 'c, N: RealField> SystemBundle<'a, 'b> for TransformBundle<'c, N> {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            HierarchySystem::<Parent>::new(),
            "parent_hierarchy_system",
            self.dep,
        );
        builder.add(
            TransformSystem::<N>::new(),
            "transform_system",
            &["parent_hierarchy_system"],
        );
        Ok(())
    }
}
