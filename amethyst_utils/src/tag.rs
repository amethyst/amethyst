//! Provides a small simple tag component for identifying entities.

use std::marker::PhantomData;

use amethyst_core::ecs::*;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// Tag component that can be used with a custom type to tag entities for processing
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Tag<T>
where
    T: Clone + Send + Sync + 'static,
{
    _m: PhantomData<T>,
}

impl<T> Default for Tag<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Tag { _m: PhantomData }
    }
}

/// Utility lookup for tag components
#[allow(missing_debug_implementations)]
#[derive(Derivative)]
#[derivative(Default)]
pub struct TagFinder<T>
where
    T: Clone + Send + Sync + 'static,
{
    _m: PhantomData<T>,
}

impl<T> TagFinder<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Returns the first entity found with the tag in question.
    pub fn find(&self, subworld: &mut SubWorld<'_>) -> Option<Entity> {
        <(Entity, Read<Tag<T>>)>::query()
            .iter(subworld)
            .map(|(ent, _)| *ent)
            .next()
    }
}
