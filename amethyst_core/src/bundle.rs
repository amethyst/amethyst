//! Provides a trait for adding bundles of systems to a dispatcher.

use crate::ecs::prelude::{DispatcherBuilder, World};
use amethyst_error::Error;

/// A bundle of ECS components, resources and systems.
pub trait SystemBundle<'a, 'b> {
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(
        self,
        world: &mut World,
        dispatcher: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error>;
}
