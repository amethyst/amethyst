use crate::ecs::prelude::*;
use crate::dispatcher::{
    SystemBundle,
    DispatcherBuilder,
    Stage,
};
use legion_transform::*;
use amethyst_error::Error;

#[derive(Debug, Default)]
pub struct TransformBundle;

impl SystemBundle for TransformBundle {
    fn build(self, world: &mut World, resources: &mut Resources, builder: &mut DispatcherBuilder<'_>) -> Result<(), Error> {
        hierarchy_maintenance_system::build(world, resources)
            .into_iter()
            .for_each(|system| builder.add_system(Stage::Begin, move |_, _| system));

        builder.add_system(Stage::Begin, local_to_parent_system::build);
        builder.add_system(Stage::Begin, local_to_world_system::build);
        builder.add_system(Stage::Begin, local_to_world_propagate_system::build);

        Ok(())
    }
}
