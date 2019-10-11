use super::*;
use crate::{
    legion::sync::{LegionSystems, LegionWorld},
    shred::DispatcherBuilder,
    transform::Transform,
    SystemBundle as SpecsSystemBundle, Time,
};
use amethyst_error::Error;
use legion::system::Schedulable;
use specs::{shred::ResourceId, World};

#[derive(Default)]
pub struct LegionBundle {
    systems: Vec<Box<dyn Consume>>,
    bundles: Vec<Box<dyn Consume>>,
    thread_locals: Vec<Box<dyn ThreadLocalSystem>>,
    syncers: Vec<Box<dyn sync::SyncerTrait>>,
}
impl LegionBundle {
    pub fn with_thread_local<D: ThreadLocalSystem + 'static>(mut self, system: D) -> Self {
        self.thread_locals.push(Box::new(system));

        self
    }

    pub fn with_system_desc<D: SystemDesc + 'static>(mut self, desc: D) -> Self {
        self.systems
            .push(Box::new(SystemDescWrapper(desc)) as Box<dyn Consume>);

        self
    }

    pub fn with_bundle<D: SystemBundle + 'static>(mut self, bundle: D) -> Self {
        self.bundles
            .push(Box::new(SystemBundleWrapper(bundle)) as Box<dyn Consume>);

        self
    }

    pub fn with_resource_sync<T: legion::resource::Resource>(mut self) -> Self {
        self.syncers
            .push(Box::new(sync::ResourceSyncer::<T>::default()));
        self
    }

    pub fn with_component_sync<T: Clone + legion::storage::Component + specs::Component>(
        mut self,
    ) -> Self {
        self.syncers
            .push(Box::new(sync::ComponentSyncer::<T>::default()));
        self
    }

    pub fn prepare(mut self, world: &mut LegionWorld, legion_systems: &mut LegionSystems) -> Self {
        {
            let legion_world = &mut world.world;
            for desc in self.systems.drain(..) {
                desc.consume(legion_world, legion_systems).unwrap()
            }

            for bundle in self.bundles.drain(..) {
                bundle.consume(legion_world, legion_systems).unwrap()
            }
        }

        for syncer in self.syncers.drain(..) {
            world.syncers.push(syncer);
        }

        // Core syncers
        world.add_resource_sync::<crate::Time>();
        world.add_resource_sync::<crate::ParentHierarchy>();
        world.add_resource_sync::<crate::ArcThreadPool>();
        world.add_resource_sync::<crate::frame_limiter::FrameLimiter>();
        world.add_resource_sync::<crate::Stopwatch>();

        world.add_component_sync::<crate::Transform>();
        world.add_component_sync::<crate::Hidden>();
        world.add_component_sync::<crate::HiddenPropagate>();
        // Why does this cause a crash? probably because this is cow borrow, but why is it Clone then?
        // Cloning it obviously causes a crash
        //world_store.add_component_sync::<crate::Named>();
        world.add_component_sync::<crate::Parent>();

        self
    }
}
impl<'a, 'b> SpecsSystemBundle<'a, 'b> for LegionBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        use crate::SystemDesc;

        Ok(())
    }
}
