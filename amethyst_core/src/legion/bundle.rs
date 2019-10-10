use super::*;
use crate::{
    shred::DispatcherBuilder, transform::Transform, SystemBundle as SpecsSystemBundle, Time,
};
use amethyst_error::Error;
use legion::system::Schedulable;
use specs::{shred::ResourceId, World};

#[derive(Default)]
pub struct LegionBundle {
    systems: Vec<Box<dyn SystemDesc>>,
    bundles: Vec<Box<dyn SystemBundle>>,
    syncers: Vec<Box<dyn sync::SyncerTrait>>,
}
impl LegionBundle {
    pub fn with_system<D: SystemDesc>(mut self, desc: D) -> Self {
        self.systems.push(Box::new(desc));

        self
    }

    pub fn with_bundle<D: SystemBundle + 'static>(mut self, bundle: D) -> Self {
        self.bundles.push(Box::new(bundle));

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
}
impl<'a, 'b> SpecsSystemBundle<'a, 'b> for LegionBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        use crate::SystemDesc;

        // Create the legion world
        let universe = legion::world::Universe::new();
        let mut legion_world = universe.create_world();
        let mut legion_resources = legion::resource::Resources::default();

        let mut legion_systems = sync::LegionSystems {
            game: self
                .systems
                .into_iter()
                .map(|desc| desc.build(&mut legion_world, &mut legion_resources))
                .collect(),
            ..Default::default()
        };

        for bundle in self.bundles.iter() {
            bundle.build(
                &mut legion_world,
                &mut legion_resources,
                &mut legion_systems,
            )?
        }

        let mut world_store = sync::LegionWorld {
            universe,
            world: legion_world,
            resources: legion_resources,
            syncers: self.syncers,
        };

        // Core syncers
        world_store.add_resource_sync::<crate::Time>();
        world_store.add_resource_sync::<crate::ParentHierarchy>();
        world_store.add_resource_sync::<crate::ArcThreadPool>();
        world_store.add_resource_sync::<crate::frame_limiter::FrameLimiter>();
        world_store.add_resource_sync::<crate::Stopwatch>();

        world_store.add_component_sync::<crate::Transform>();
        world_store.add_component_sync::<crate::Hidden>();
        world_store.add_component_sync::<crate::HiddenPropagate>();
        // Why does this cause a crash? probably because this is cow borrow, but why is it Clone then?
        // Cloning it obviously causes a crash
        //world_store.add_component_sync::<crate::Named>();
        world_store.add_component_sync::<crate::Parent>();

        world.insert(legion_systems);
        world.insert(world_store);

        builder.add(
            sync::LegionSyncEntitySystemDesc::default().build(world),
            "LegionSyncEntitySystem",
            &[],
        );

        Ok(())
    }
}
