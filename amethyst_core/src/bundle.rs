use specs::prelude::{DispatcherBuilder, World};

error_chain!{}

/// A bundle of ECS components, resources and systems.
pub trait ECSBundle<'a, 'b> {
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(
        self,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>>;
}
