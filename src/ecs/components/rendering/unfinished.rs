//! Components for the rendering processor.

use ecs::{Component, HashMapStorage};
use renderer::{Renderer, Result};
use super::mesh::*;
use super::material::*;

/// Transform builder into `Unfinshed<T>`
pub trait IntoUnfinished {
    /// Output when finished
    type Output: Component;

    /// Transform
    fn unfinished(self) -> Unfinished<Self::Output>;
}

/// Wrapper for boxed `FactoryBuilder` with `Output: Component`
/// Use this to put `FactoryBuilder` as `Component` to entity
/// so relevant `System` may finish it
/// and attach result to the same entity
pub struct Unfinished<T: Component>(Box<ComponentBuilder<Output=T> + Send + Sync>);

impl<T> Unfinished<T>
    where T: Component
{
    /// Finish this component
    /// Intended to be used by relevant `System`
    pub(crate) fn finish(self, renderer: &mut Renderer) -> Result<T> {
        self.0.build(renderer)
    }
}

impl<T> Unfinished<T>
    where T: Component
{
    /// Wrap builder into `Unfinished`
    /// making possible to attach the builder to `Entity`
    pub(crate) fn new<B>(builder: B) -> Self
        where B: ComponentBuilder<Output=T> + Send + Sync + 'static
    {
        Unfinished(Box::new(builder))
    }
}

impl<T> Component for Unfinished<T>
    where T: Component
{
    type Storage = HashMapStorage<Self>;
}

pub(crate) trait ComponentBuilder {
    type Output: Component;
    fn build(self: Box<Self>, renderer: &mut Renderer) -> Result<Self::Output>;
}