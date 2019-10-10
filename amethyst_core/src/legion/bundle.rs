use super::*;
use crate::{shred::DispatcherBuilder, transform::Transform, SystemBundle, SystemDesc, Time};
use amethyst_error::Error;
use legion::system::Schedulable;
use specs::{shred::ResourceId, World};

#[derive(Default)]
pub struct LegionBundle {
    systems: Vec<Box<dyn LegionSystemDesc>>,
    syncers: Vec<Box<dyn sync::SyncerTrait>>,
}
impl LegionBundle {
    pub fn with_system<D: LegionSystemDesc>(mut self, desc: D) -> Self {
        self.systems.push(Box::new(desc));

        self
    }

    pub fn with_sync<T: legion::resource::Resource>(mut self) -> Self {
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
impl<'a, 'b> SystemBundle<'a, 'b> for LegionBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        // Create the legion world
        let universe = legion::world::Universe::new();
        let mut legion_world = universe.create_world();
        let mut legion_resources = legion::resource::Resources::default();

        world.insert(sync::LegionSystems(
            self.systems
                .into_iter()
                .map(|desc| desc.build(&mut legion_world))
                .collect(),
        ));

        world.insert(sync::LegionWorld {
            universe,
            world: legion_world,
            resources: legion_resources,
            syncers: self.syncers,
        });

        builder.add(
            sync::LegionSyncEntitySystemDesc::default().build(world),
            "LegionSyncEntitySystem",
            &[],
        );

        Ok(())
    }
}
