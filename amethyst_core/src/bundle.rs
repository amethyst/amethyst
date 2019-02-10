//! Provides a trait for adding bundles of systems to a dispatcher.

use amethyst_error::Error;
use specs::prelude::DispatcherBuilder;

/// A bundle of ECS components, resources and systems.
pub trait SystemBundle<'a, 'b> {
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(self, dispatcher: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error>;
}
