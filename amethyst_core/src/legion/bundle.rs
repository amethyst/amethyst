use super::*;
use crate::{
    legion::{dispatcher::DispatcherBuilder, Allocators, LegionState},
    transform::Transform,
    SystemBundle as SpecsSystemBundle, Time,
};
use amethyst_error::Error;
use legion::system::Schedulable;
use specs::{shred::ResourceId, World};

#[derive(Default)]
pub struct LegionSyncer {
    syncers: Vec<Box<dyn sync::SyncerTrait>>,
}
impl LegionSyncer {
    pub fn prepare(mut self, state: &mut LegionState, dispatcher: &mut DispatcherBuilder) {
        for syncer in self.syncers.drain(..) {
            state.syncers.push(syncer);
        }

        // state.add_resource_sync::<Allocators>();

        // Core syncers
        state.add_resource_sync::<crate::Time>();
        state.add_resource_sync::<crate::ParentHierarchy>();
        state.add_resource_sync::<crate::ArcThreadPool>();
        state.add_resource_sync::<crate::frame_limiter::FrameLimiter>();
        state.add_resource_sync::<crate::Stopwatch>();

        state.add_component_sync::<crate::Transform>();
        state.add_component_sync::<crate::Hidden>();
        state.add_component_sync::<crate::HiddenPropagate>();
        // Why does this cause a crash? probably because this is cow borrow, but why is it Clone then?
        // Cloning it obviously causes a crash
        //world_store.add_component_sync::<crate::Named>();
        state.add_component_sync::<crate::Parent>();
    }
}
