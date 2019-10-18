//! TODO: doc
//!

pub mod dispatcher;
pub mod sync;
pub mod temp;

pub use dispatcher::{ConsumeDesc, Dispatcher, DispatcherBuilder, Stage};
pub use legion::{prelude::*, *};
pub use sync::{ComponentSyncer, ComponentSyncerWith, ResourceSyncer, SyncDirection, SyncerTrait};

pub trait SystemDesc: 'static {
    fn build(mut self, world: &mut legion::world::World) -> Box<dyn legion::schedule::Schedulable>;
}

pub trait ThreadLocal {
    fn run(&mut self, world: &mut World);
    fn dispose(self, world: &mut World);
}

pub trait ThreadLocalDesc: 'static {
    fn build(mut self, world: &mut legion::world::World) -> Box<dyn ThreadLocal>;
}

pub trait SystemBundle {
    fn build(
        self,
        world: &mut legion::world::World,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error>;
}

pub struct DispatcherSystemDesc<B>(Stage, B);
impl<B: SystemDesc> ConsumeDesc for DispatcherSystemDesc<B> {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        dispatcher: &mut Dispatcher,
        _: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        dispatcher
            .stages
            .get_mut(&self.0)
            .unwrap()
            .push(self.1.build(world));
        Ok(())
    }
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

pub struct DispatcherThreadLocalDesc<B>(B);
impl<B: ThreadLocalDesc> ConsumeDesc for DispatcherThreadLocalDesc<B> {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        dispatcher: &mut Dispatcher,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        dispatcher.thread_locals.push(self.0.build(world));
        Ok(())
    }
}

pub struct DispatcherThreadLocal<B>(B);
impl<B: 'static + ThreadLocal> ConsumeDesc for DispatcherThreadLocal<B> {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        dispatcher: &mut Dispatcher,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        dispatcher.thread_locals.push(Box::new(self.0));
        Ok(())
    }
}

pub struct DispatcherSystem<F>(Stage, F);
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
        dispatcher
            .stages
            .get_mut(&self.0)
            .unwrap()
            .push((self.1)(world));
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
        T::Storage: Default,
    {
        self.syncers.push(Box::new(ComponentSyncer::<T>::default()));
    }

    pub fn add_component_sync_with<S, L, F>(&mut self, f: F)
    where
        S: Send + Sync + specs::Component,
        L: legion::storage::Component,
        F: 'static
            + Fn(SyncDirection, Option<&mut S>, Option<&mut L>) -> (Option<S>, Option<L>)
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

        // state.add_resource_sync::<Allocators>();

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
        // Why does this cause a crash? probably because this is cow borrow, but why is it Clone then?
        // Cloning it obviously causes a crash
        //world_store.add_component_sync::<crate::Named>();
        state.add_component_sync::<crate::Parent>();
    }
}
