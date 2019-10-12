use crate::{
    legion::{SystemDesc, ThreadLocalSystem},
    transform::Transform,
};
use bimap::BiMap;
use derivative::Derivative;
use legion::{command::CommandBuffer, event::ListenerId};
use shrinkwraprs::Shrinkwrap;
use smallvec::SmallVec;
use specs::{
    shrev::ReaderId,
    storage::{ComponentEvent, GenericWriteStorage},
    Builder, Component, DenseVecStorage, Entities, FlaggedStorage, Join, LazyUpdate, NullStorage,
    Read, ReadExpect, ReadStorage, RunNow, System, SystemData, WorldExt, Write, WriteExpect,
    WriteStorage,
};
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

pub struct LegionWorld {
    pub universe: legion::world::Universe,
    pub world: legion::world::World,
    pub syncers: Vec<Box<dyn SyncerTrait>>,
}
impl LegionWorld {
    pub fn add_resource_sync<T: legion::resource::Resource>(&mut self) {
        self.syncers.push(Box::new(ResourceSyncer::<T>::default()));
    }

    pub fn add_component_sync<T: Clone + legion::storage::Component + specs::Component>(&mut self) {
        self.syncers.push(Box::new(ComponentSyncer::<T>::default()));
    }
}

#[derive(Default)]
pub struct LegionSystems {
    pub thread_locals: Vec<Box<dyn ThreadLocalSystem>>,
    pub game: Vec<Box<dyn legion::system::Schedulable>>,
    pub render: Vec<Box<dyn legion::system::Schedulable>>,
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
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SpecsTag;

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct LegionTag;
impl Component for LegionTag {
    type Storage = NullStorage<Self>;
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
        legion_world: &mut legion::world::World,
    ) {
        let map = bimap.read().unwrap();
        match direction {
            SyncDirection::SpecsToLegion => {
                for (entity, component) in (entities, storage).join() {
                    if let Some(legion_entity) = map.get_by_right(&entity) {
                        let exists = legion_world
                            .get_component_mut::<T>(*legion_entity)
                            .is_some();
                        if exists {
                            *legion_world.get_component_mut::<T>(*legion_entity).unwrap() =
                                (*component).clone();
                        } else {
                            legion_world.add_component(*legion_entity, (*component).clone());
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
                            storage.insert(*specs_entity, (*component).clone()).unwrap();
                        }
                    }
                }
            }
        }
    }
}
impl<T> Syncable<T> for T where T: Clone + legion::storage::Component + specs::Component {}

pub trait SyncerTrait: 'static + Send + Sync {
    fn sync(
        &self,
        world: &mut specs::World,
        legion_world: &mut LegionWorld,
        direction: SyncDirection,
    );
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ComponentSyncer<T>(PhantomData<T>);
impl<T> SyncerTrait for ComponentSyncer<T>
where
    T: Clone + legion::storage::Component + specs::Component,
{
    fn sync(
        &self,
        world: &mut specs::World,
        legion_world: &mut LegionWorld,
        direction: SyncDirection,
    ) {
        let (bimap, entities, mut storage) = <(
            Read<'_, EntitiesBimapRef>,
            Entities<'_>,
            WriteStorage<'_, T>,
        )>::fetch(world);

        T::sync(
            direction,
            bimap.clone(),
            &entities,
            &mut storage,
            &mut legion_world.world,
        );
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ResourceSyncer<T>(PhantomData<T>);
impl<T> SyncerTrait for ResourceSyncer<T>
where
    T: legion::resource::Resource,
{
    fn sync(
        &self,
        world: &mut specs::World,
        legion_world: &mut LegionWorld,
        direction: SyncDirection,
    ) {
        move_resource::<T>(world, legion_world, direction);
    }
}

pub fn move_resource<T: legion::resource::Resource>(
    world: &mut specs::World,
    legion_world: &mut LegionWorld,
    direction: SyncDirection,
) {
    match direction {
        SyncDirection::SpecsToLegion => {
            if let Some(resource) = world.remove::<T>() {
                legion_world.world.resources.insert(resource);
            }
        }
        SyncDirection::LegionToSpecs => {
            let resource = legion_world.world.resources.remove::<T>();
            if let Some(resource) = resource {
                world.insert(resource);
            }
        }
    }
}

pub fn dispatch_legion(
    specs_world: &mut specs::World,
    legion_world: &mut LegionWorld,
    legion_systems: &mut LegionSystems,
) {
    let syncers = legion_world.syncers.drain(..).collect::<Vec<_>>();

    syncers
        .iter()
        .for_each(|s| s.sync(specs_world, legion_world, SyncDirection::SpecsToLegion));

    {
        let world = &legion_world.world;

        let mut game_stage =
            legion::system::StageExecutor::new(&mut legion_systems.game).execute(world);

        let mut render_stage =
            legion::system::StageExecutor::new(&mut legion_systems.render).execute(world);
    }

    syncers
        .iter()
        .for_each(|s| s.sync(specs_world, legion_world, SyncDirection::LegionToSpecs));

    legion_world.syncers.extend(syncers.into_iter());
}

pub fn setup(
    specs_world: &mut specs::World,
    legion_world: &mut LegionWorld,
    legion_systems: &mut LegionSystems,
) {
    let entity_map = Arc::new(RwLock::new(
        BiMap::<legion::entity::Entity, specs::Entity>::new(),
    ));
    legion_world.world.resources.insert(entity_map.clone());
    specs_world.insert(entity_map.clone());
}

pub fn sync_entities(
    specs_world: &mut specs::World,
    legion_world: &mut LegionWorld,
    legion_listener_id: ListenerId,
) {
    use smallvec::SmallVec;

    let mut new_entities = SmallVec::<[legion::entity::Entity; 512]>::new();

    let specs_entities = {
        let entity_bimap = legion_world
            .world
            .resources
            .get::<EntitiesBimapRef>()
            .unwrap();
        let mut map = entity_bimap.read().unwrap();

        (&<(Entities<'_>)>::fetch(specs_world))
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
        let new_legion = legion_world
            .world
            .insert((SpecsTag,), specs_entities.clone())
            .iter()
            .enumerate()
            .map(|(i, legion_entity)| {
                log::trace!(
                    "{} - legion:[{:?}] = specs:[{:?}]",
                    i,
                    legion_entity,
                    specs_entities[i].0.specs_entity
                );
                (i, *legion_entity)
            })
            .collect::<Vec<_>>();

        let entity_bimap = legion_world
            .world
            .resources
            .get_mut::<EntitiesBimapRef>()
            .unwrap();
        let mut map = entity_bimap.write().unwrap();
        for (i, entity) in new_legion {
            map.insert(entity, specs_entities[i].0.specs_entity);
        }
    }

    while let Ok(event) = legion_world.world.entity_channel().read(legion_listener_id) {
        let entity_bimap = legion_world
            .world
            .resources
            .get::<EntitiesBimapRef>()
            .unwrap();
        let map = entity_bimap.read().unwrap();
        match event {
            legion::event::EntityEvent::Created(e) => {
                if !map.contains_left(&e) {
                    new_entities.push(e);
                }
            }
            legion::event::EntityEvent::Deleted(e) => if map.contains_left(&e) {},
        }
    }

    if !new_entities.is_empty() {
        let entity_bimap = legion_world
            .world
            .resources
            .get_mut::<EntitiesBimapRef>()
            .unwrap();
        let mut map = entity_bimap.write().unwrap();
        new_entities.iter().for_each(|e| {
            let specs_entity = specs_world.create_entity().with(LegionTag).build();
            map.insert(*e, specs_entity);
        });
    }
}
