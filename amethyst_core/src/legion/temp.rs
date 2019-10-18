use crate::legion::{
    dispatcher::{Dispatcher, Stage},
    sync::SyncDirection,
    LegionState,
};
use bimap::BiMap;
use std::sync::{Arc, RwLock};

pub fn dispatch_legion(
    specs_world: &mut specs::World,
    legion_state: &mut LegionState,
    dispatcher: &mut Dispatcher,
) {
    let syncers = legion_state.syncers.drain(..).collect::<Vec<_>>();

    syncers
        .iter()
        .for_each(|s| s.sync(specs_world, legion_state, SyncDirection::SpecsToLegion));

    dispatcher.run(Stage::Begin, &mut legion_state.world);
    dispatcher.run(Stage::Logic, &mut legion_state.world);
    dispatcher.run(Stage::Render, &mut legion_state.world);
    dispatcher.run(Stage::ThreadLocal, &mut legion_state.world);
    syncers
        .iter()
        .for_each(|s| s.sync(specs_world, legion_state, SyncDirection::LegionToSpecs));

    legion_state.syncers.extend(syncers.into_iter());
}

pub fn setup(specs_world: &mut specs::World, legion_state: &mut LegionState) {
    let entity_map = Arc::new(RwLock::new(
        BiMap::<legion::entity::Entity, specs::Entity>::new(),
    ));
    legion_state.world.resources.insert(entity_map.clone());
    specs_world.insert(entity_map.clone());

    //legion_state.world.resources.insert(Allocators::default());
}
