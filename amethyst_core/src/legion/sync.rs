use crate::{transform::Transform, SystemDesc};
use bimap::BiMap;
use legion::{command::CommandBuffer, event::ListenerId};
use specs::{
    shrev::ReaderId,
    storage::{ComponentEvent, GenericWriteStorage},
    Component, DenseVecStorage, Entities, FlaggedStorage, Join, NullStorage, Read, ReadExpect,
    ReadStorage, RunNow, System, SystemData, WorldExt, Write, WriteExpect, WriteStorage,
};
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

pub struct LegionWorld {
    universe: legion::world::Universe,
    world: legion::world::World,
    resources: legion::resource::Resources,
}

#[derive(Default)]
pub struct LegionSyncFlagComponent;
impl Component for LegionSyncFlagComponent {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SpecsEntityComponent {
    specs_entity: specs::Entity,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SpecsTag;

pub trait LegionSystemDesc {
    fn build(&mut self, world: &mut legion::world::World) -> Box<dyn legion::system::Schedulable>;
}

type EntitiesBimapRef = Arc<RwLock<BiMap<legion::entity::Entity, specs::Entity>>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SyncDirection {
    LegionToSpecs,
    SpecsToLegion,
}

pub trait Syncable<T>
where
    T: Clone + legion::storage::Component + specs::Component + Send + Sync,
{
    fn sync<'a>(
        direction: SyncDirection,
        bimap: EntitiesBimapRef,
        entities: &Entities<'a>,
        storage: &mut WriteStorage<'a, T>,
        legion_world: &legion::world::World,
        command_buffer: &mut CommandBuffer,
    ) {
        let map = bimap.read().unwrap();

        match direction {
            SyncDirection::SpecsToLegion => {
                for (entity, component) in (entities, storage).join() {
                    if let Some(legion_entity) = map.get_by_right(&entity) {
                        if let Some(mut legion_component) =
                            legion_world.get_component_mut::<T>(*legion_entity)
                        {
                            *legion_component = (*component).clone();
                        } else {
                            command_buffer.add_component(*legion_entity, (*component).clone())
                        }
                    }
                }
            }
            SyncDirection::LegionToSpecs => {
                use legion::prelude::*;
                let mut query = <(Read<T>)>::query();
                for (entity, component) in query.iter_entities(legion_world) {
                    if let Some(specs_entity) = map.get_by_left(&entity) {
                        if let Some(specs_component) = storage.get_mut(*specs_entity) {
                            *specs_component = (*component).clone();
                        } else {
                            storage.insert(*specs_entity, (*component).clone());
                        }
                    }
                }
            }
        }
    }
}
impl<T> Syncable<T> for T where T: Clone + legion::storage::Component + specs::Component {}

pub struct ComponentSyncSystem<T> {
    _marker: PhantomData<T>,
    direction: SyncDirection,
}
impl<T> ComponentSyncSystem<T>
where
    T: Clone + legion::storage::Component + specs::Component,
{
    pub fn new(direction: SyncDirection) -> Self {
        Self {
            direction,
            _marker: Default::default(),
        }
    }
}

impl<'a, T> System<'a> for ComponentSyncSystem<T>
where
    T: Clone + legion::storage::Component + specs::Component,
{
    type SystemData = (
        Read<'a, EntitiesBimapRef>,
        Entities<'a>,
        WriteStorage<'a, T>,
        ReadExpect<'a, LegionWorld>,
        Write<'a, CommandBuffer>,
    );
    fn run(
        &mut self,
        (bimap, entities, mut storage, legion_world, mut command_buffer): Self::SystemData,
    ) {
        T::sync(
            self.direction,
            bimap.clone(),
            &entities,
            &mut storage,
            &legion_world.world,
            &mut command_buffer,
        );
    }
}

pub struct LegionSyncEntitySystem {
    pub legion_listener_id: ListenerId,
}

impl<'a> System<'a> for LegionSyncEntitySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, LegionWorld>,
        WriteExpect<'a, EntitiesBimapRef>,
        Write<'a, CommandBuffer>,
    );
    fn run(
        &mut self,
        (entities, mut legion_world, mut entity_bimap, mut command_buffer): Self::SystemData,
    ) {
        let specs_entities = {
            let mut map = entity_bimap.read().unwrap();

            (&entities)
                .join()
                .filter_map(|(entity)| {
                    if !map.contains_right(&entity) {
                        Some((SpecsEntityComponent {
                            specs_entity: entity,
                        },))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        if !specs_entities.is_empty() {
            let mut map = entity_bimap.write().unwrap();

            legion_world
                .world
                .insert((SpecsTag,), specs_entities.clone())
                .iter()
                .enumerate()
                .for_each(|(i, legion_entity)| {
                    log::trace!(
                        "{} - Entity [l={:?},s={:?} created in legion, inserting to map",
                        i,
                        legion_entity,
                        specs_entities[i].0.specs_entity
                    );
                    map.insert(*legion_entity, specs_entities[i].0.specs_entity);
                });
        }

        while let Ok(_) = legion_world
            .world
            .entity_channel()
            .read(self.legion_listener_id)
        {}

        // Flush the command buffer for modifications
        command_buffer.write(&mut legion_world.world);
    }
}

#[derive(Default)]
pub struct LegionSyncEntitySystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, LegionSyncEntitySystem> for LegionSyncEntitySystemDesc {
    fn build(self, world: &mut specs::World) -> LegionSyncEntitySystem {
        <LegionSyncEntitySystem as System<'_>>::SystemData::setup(world);

        let entity_map = Arc::new(RwLock::new(
            BiMap::<legion::entity::Entity, specs::Entity>::new(),
        ));
        world.insert(entity_map.clone());

        // Create the legion world
        let universe = legion::world::Universe::new();
        let mut legion_world = universe.create_world();
        let mut legion_resources = legion::resource::Resources::default();
        legion_resources.insert(entity_map.clone());

        let sync_system = SyncSystemLegionDesc::default().build(&mut legion_world);

        let legion_listener_id = legion_world.entity_channel().bind_listener(1024);

        world.insert(LegionWorld {
            universe,
            world: legion_world,
            resources: legion_resources,
        });
        world.insert(vec![sync_system]);

        LegionSyncEntitySystem { legion_listener_id }
    }
}

pub struct LegionDispatcherSystem {}
impl<'a> System<'a> for LegionDispatcherSystem {
    type SystemData = (
        WriteExpect<'a, Vec<Box<dyn legion::system::Schedulable>>>,
        ReadExpect<'a, LegionWorld>,
    );
    fn run(&mut self, (mut legion_systems, legion_world): Self::SystemData) {
        let resources = &legion_world.resources;
        let world = &legion_world.world;

        let mut stage =
            legion::system::StageExecutor::new(&mut legion_systems).execute(resources, world);
    }
}

#[derive(Default)]
pub struct LegionDispatcherSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, LegionDispatcherSystem> for LegionDispatcherSystemDesc {
    fn build(self, world: &mut specs::World) -> LegionDispatcherSystem {
        <LegionDispatcherSystem as System<'_>>::SystemData::setup(world);

        LegionDispatcherSystem {}
    }
}

#[derive(Default)]
pub struct SyncSystemLegionDesc;
impl LegionSystemDesc for SyncSystemLegionDesc {
    fn build(&mut self, world: &mut legion::world::World) -> Box<dyn legion::system::Schedulable> {
        use legion::prelude::*;

        SystemBuilder::<()>::new("Test")
            .with_query(<(Read<Transform>)>::query())
            .build(move |commands, world, _resource, query| {
                query.iter_entities().for_each(|(entity, transform)| {});
            })
    }
}
