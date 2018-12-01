//! Provides a trait for adding bundles of systems to a dispatcher.

use crate::SimpleDispatcherBuilder;

error_chain!{}

/// A bundle of ECS components, resources and systems.
pub trait SystemBundle<'a, 'b, 'c, D>
where
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(self, dispatcher: &mut D) -> Result<()>;
}
