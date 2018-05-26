//! Traits for bundles of entities, components and/or systems.
use failure::Error;
use specs::prelude::DispatcherBuilder;

/// A bundle of ECS components, resources and systems.
// TODO consider associated error type
pub trait SystemBundle<'a, 'b> {
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(self, dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error>;
}
