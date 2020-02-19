pub mod sync;

use crate::legion::{dispatcher::SystemBundle, *};
use amethyst_error::Error;
pub use legion_transform::components;
use legion_transform::*;

#[derive(Debug, Default)]
pub struct Syncer;
impl LegionSyncBuilder for Syncer {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        state: &mut LegionState,
        dispatcher: &mut DispatcherBuilder<'_>,
    ) {
        state.add_sync(sync::TransformSyncer::default());
    }
}

#[derive(Debug, Default)]
pub struct TransformBundle;
impl SystemBundle for TransformBundle {
    fn build(
        mut self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        hierarchy_maintenance_system::build(world, resources)
            .into_iter()
            .for_each(|system| builder.add_system(Stage::Begin, move |_, _| system));

        builder.add_system(Stage::Begin, local_to_parent_system::build);
        builder.add_system(Stage::Begin, local_to_world_system::build);
        builder.add_system(Stage::Begin, local_to_world_propagate_system::build);

        Ok(())
    }
}
