use crate::ecs::*;
use amethyst_error::Error;

pub use legion_transform::prelude::*;

/// Bundle to add the transformation systems.
#[derive(Debug, Default)]
pub struct TransformBundle;

impl SystemBundle for TransformBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder.with_system(missing_previous_parent_system::build());
        builder.with_system(parent_update_system::build());
        builder.with_system(local_to_parent_system::build());
        builder.with_system(local_to_world_system::build());
        builder.with_system(local_to_world_propagate_system::build());

        Ok(())
    }
}
