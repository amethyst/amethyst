//! TODO: doc
//!

use std::marker::PhantomData;

pub mod dispatcher;
pub mod sync;
pub mod temp;

pub mod transform;

pub use dispatcher::{
    ConsumeDesc, Dispatcher, DispatcherBuilder, IntoRelativeStage, RelativeStage, Stage,
    ThreadLocal,
};
pub use legion::{prelude::*, *};
pub use sync::{
    ComponentSyncer, ComponentSyncerWith, EntitiesBimapRef, ResourceSyncer, SyncDirection,
    SyncerTrait,
};

pub trait SystemBundle {
    fn build(
        self,
        world: &mut legion::world::World,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error>;
}
pub struct DispatcherSystemBundle<B>(B);
impl<B: SystemBundle> ConsumeDesc for DispatcherSystemBundle<B> {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        _: &mut Dispatcher,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        self.0.build(world, builder)?;
        Ok(())
    }
}

pub struct DispatcherSystem<F>(RelativeStage, F);
impl<F> ConsumeDesc for DispatcherSystem<F>
where
    F: FnOnce(&mut World) -> Box<dyn Schedulable>,
{
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        dispatcher: &mut Dispatcher,
        _: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        let sys = (self.1)(world);

        dispatcher
            .stages
            .entry(self.0)
            .or_insert_with(Vec::default)
            .push(sys);

        Ok(())
    }
}

pub struct DispatcherThreadLocalSystem<F>(F);
impl<F> ConsumeDesc for DispatcherThreadLocalSystem<F>
where
    F: FnOnce(&mut World) -> Box<dyn Runnable>,
{
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        dispatcher: &mut Dispatcher,
        _: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        dispatcher.thread_local_systems.push((self.0)(world));
        Ok(())
    }
}

pub struct DispatcherThreadLocal<F>(F);
impl<F> ConsumeDesc for DispatcherThreadLocal<F>
where
    F: FnOnce(&mut World) -> Box<dyn ThreadLocal>,
{
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        dispatcher: &mut Dispatcher,
        _: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        dispatcher.thread_locals.push((self.0)(world));
        Ok(())
    }
}

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
    pub syncers: Vec<Box<dyn SyncerTrait>>,
}

impl LegionState {
    pub fn add_resource_sync<T: legion::resource::Resource>(&mut self) {
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
        for syncer in self.syncers.drain(..) {
            state.syncers.push(syncer);
        }

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
