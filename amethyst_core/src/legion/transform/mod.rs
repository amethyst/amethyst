pub mod sync;

use crate::legion::*;
use amethyst_error::Error;
use legion_transform::*;

pub use legion_transform::components;

#[derive(Default)]
pub struct Syncer;
impl LegionSyncBuilder for Syncer {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        state: &mut LegionState,
        dispatcher: &mut DispatcherBuilder,
    ) {
        state.add_sync(sync::TransformSyncer::default());
    }
}

#[derive(Default)]
pub struct TransformBundle;
impl SystemBundle for TransformBundle {
    fn build(mut self, world: &mut World, builder: &mut DispatcherBuilder) -> Result<(), Error> {
        hierarchy_maintenance_system::build(world)
            .into_iter()
            .for_each(|system| builder.add_system(Stage::Begin, move |_| system));

        builder.add_system(Stage::Begin, local_to_parent_system::build);
        builder.add_system(Stage::Begin, local_to_world_system::build);
        builder.add_system(Stage::Begin, local_to_world_propagate_system::build);

        Ok(())
    }
}
