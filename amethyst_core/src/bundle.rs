use std::result::Result as StdResult;

use specs::{World, DispatcherBuilder};
use specs::error::BoxedErr;

/// Bundle result type.
pub type Result<T> = StdResult<T, BoxedErr>;

/// A bundle of ECS components, resources and systems.
pub trait ECSBundle<'a, 'b> {
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(
        self,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>
    ) -> Result<DispatcherBuilder<'a, 'b>>;
}
