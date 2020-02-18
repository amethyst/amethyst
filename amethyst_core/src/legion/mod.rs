//! TODO: doc
//!

use std::marker::PhantomData;

pub mod dispatcher;
pub mod sync;
pub mod temp;

pub mod transform;

pub use dispatcher::{
    ConsumeDesc, Dispatcher, DispatcherBuilder, DispatcherData, RelativeStage, Stage,
};
pub use legion::{prelude::*, *};
pub use sync::{
    ComponentSyncer, ComponentSyncerWith, EntitiesBimapRef, ResourceSyncer, SyncDirection,
    SyncerTrait,
};

pub trait LegionSyncBuilder {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        state: &mut LegionState,
        dispatcher: &mut DispatcherBuilder,
    );
}

pub struct LegionState {
    pub universe: legion::world::Universe,
    pub world: legion::world::World,
    pub resources: legion::prelude::Resources,
    pub syncers: Vec<Box<dyn SyncerTrait>>,
}

impl LegionState {
    pub fn add_sync<T: SyncerTrait>(&mut self, syncer: T) {
        self.syncers.push(Box::new(syncer));
    }

    pub fn add_resource_sync<T: legion::systems::resource::Resource>(&mut self) {
        self.syncers.push(Box::new(ResourceSyncer::<T>::default()));
    }

    pub fn add_component_sync<T>(&mut self)
    where
        T: Clone + legion::storage::Component + specs::Component,
        <T as specs::Component>::Storage: Default,
    {
        self.syncers.push(Box::new(ComponentSyncer::<T>::default()));
    }

    pub fn add_component_sync_with<S, L, F>(&mut self, f: F)
    where
        S: Send + Sync + specs::Component,
        <S as specs::Component>::Storage: Default,
        L: legion::storage::Component,
        F: 'static
            + Fn(
                SyncDirection,
                EntitiesBimapRef,
                Option<&mut S>,
                Option<&mut L>,
            ) -> (Option<S>, Option<L>)
            + Send
            + Sync,
    {
        self.syncers
            .push(Box::new(ComponentSyncerWith::<S, L, F>::new(f)));
    }
}

#[derive(Default)]
pub struct Syncer {
    syncers: Vec<Box<dyn sync::SyncerTrait>>,
}
impl LegionSyncBuilder for Syncer {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        state: &mut LegionState,
        dispatcher: &mut DispatcherBuilder,
    ) {
        use specs::WorldExt;

        specs_world.register::<sync::LegionTag>();

        for syncer in self.syncers.drain(..) {
            state.syncers.push(syncer);
        }

        state.add_sync(transform::sync::TransformSyncer::default());

        state.add_resource_sync::<crate::allocators::Allocators>();

        // Core syncers
        state.add_resource_sync::<crate::Time>();
        state.add_resource_sync::<crate::ParentHierarchy>();
        state.add_resource_sync::<crate::ArcThreadPool>();
        state.add_resource_sync::<crate::frame_limiter::FrameLimiter>();
        state.add_resource_sync::<crate::Stopwatch>();

        state.add_resource_sync::<crate::allocators::Allocators>();

        state.add_component_sync::<crate::Transform>();
        state.add_component_sync::<crate::Hidden>();
        state.add_component_sync::<crate::HiddenPropagate>();
        //world_store.add_component_sync::<crate::Named>();
        state.add_component_sync::<crate::Parent>();
    }
}
