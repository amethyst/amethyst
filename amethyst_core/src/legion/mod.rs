//! TODO: doc
//!

pub mod bundle;
pub mod dispatcher;
pub mod sync;
pub mod temp;

pub use dispatcher::{ConsumeDesc, Dispatcher, DispatcherBuilder, Stage};
pub use legion::{prelude::*, *};
pub use sync::{ComponentSyncer, ResourceSyncer, SyncerTrait};

pub trait SystemDesc: 'static {
    fn build(mut self, world: &mut legion::world::World) -> Box<dyn legion::system::Schedulable>;
}

pub trait SystemBundle {
    fn build(
        self,
        world: &mut legion::world::World,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error>;
}

pub struct DispatcherSystemDesc<B>(Stage, B)
where
    B: SystemDesc;

impl<B: SystemDesc> ConsumeDesc for DispatcherSystemDesc<B> {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        stages: &mut Dispatcher,
        _: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        println!("Stages = {:?}", stages.stages.len());
        stages
            .stages
            .get_mut(&self.0)
            .unwrap()
            .push(self.1.build(world));
        Ok(())
    }
}

pub struct DispatcherSystemBundle<B>(B)
where
    B: SystemBundle;

impl<B: SystemBundle> ConsumeDesc for DispatcherSystemBundle<B> {
    fn consume(
        self: Box<Self>,
        world: &mut legion::world::World,
        _: &mut Dispatcher,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        self.0.build(world, builder)
    }
}

pub trait ThreadLocalSystem {
    fn run(&mut self, world: &mut World);
    fn dispose(self, world: &mut World);
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

    pub fn add_component_sync<T: Clone + legion::storage::Component + specs::Component>(&mut self) {
        self.syncers.push(Box::new(ComponentSyncer::<T>::default()));
    }
}
