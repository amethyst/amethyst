use crate::{
    legion::{
        dispatcher::{Dispatcher, Stage},
        LegionState,
    },
    transform::Transform,
};
use bimap::BiMap;
use derivative::Derivative;
use legion::{command::CommandBuffer, entity::Entity as LegionEntity, event::ListenerId};
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

pub type EntitiesBimapRef = Arc<RwLock<BiMap<legion::entity::Entity, specs::Entity>>>;

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
        legion_state: &mut legion::world::World,
    ) {
        log::trace!("{:?} - {}", direction, std::any::type_name::<T>());

        let map = bimap.read().unwrap();
        match direction {
            SyncDirection::SpecsToLegion => {
                for (entity, component) in (entities, storage).join() {
                    if let Some(legion_entity) = map.get_by_right(&entity) {
                        let exists = legion_state
                            .get_component_mut::<T>(*legion_entity)
                            .is_some();
                        if exists {
                            *legion_state.get_component_mut::<T>(*legion_entity).unwrap() =
                                (*component).clone();
                        } else {
                            legion_state.add_component(*legion_entity, (*component).clone());
                        }
                    }
                }
            }
            SyncDirection::LegionToSpecs => {
                use legion::prelude::*;
                let mut query = <(Read<T>)>::query();
                for (entity, component) in query.iter_entities(legion_state) {
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
        legion_state: &mut LegionState,
        direction: SyncDirection,
    );

    fn setup(&self, world: &mut specs::World);
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ComponentSyncer<T>(PhantomData<T>);
impl<T> SyncerTrait for ComponentSyncer<T>
where
    T: Clone + legion::storage::Component + specs::Component,
    T::Storage: Default,
{
    fn setup(&self, world: &mut specs::World) {
        log::trace!(
            "Creating component synchronizer for: {:?}",
            std::any::type_name::<T>()
        );
        world.register::<T>();
    }

    fn sync(
        &self,
        world: &mut specs::World,
        legion_state: &mut LegionState,
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
            &mut legion_state.world,
        );
    }
}

pub struct ComponentSyncerWith<S, L, F>(F, PhantomData<(S, L)>);
impl<S, L, F> ComponentSyncerWith<S, L, F>
where
    S: specs::Component + Send + Sync,
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
    pub fn new(f: F) -> Self {
        Self(f, Default::default())
    }
}
impl<S, L, F> SyncerTrait for ComponentSyncerWith<S, L, F>
where
    S: specs::Component + Send + Sync,
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
    fn setup(&self, world: &mut specs::World) {
        log::trace!(
            "Creating closure component synchronizer for: {:?} - {:?}",
            std::any::type_name::<S>(),
            std::any::type_name::<L>()
        );
        world.register::<S>();
    }

    fn sync(
        &self,
        world: &mut specs::World,
        legion_state: &mut LegionState,
        direction: SyncDirection,
    ) {
        let (bimap, entities, mut storage) = <(
            Read<'_, EntitiesBimapRef>,
            Entities<'_>,
            WriteStorage<'_, S>,
        )>::fetch(world);

        let mut new_legion: SmallVec<[(LegionEntity, L); 64]> = SmallVec::default();
        let mut new_specs: SmallVec<[(specs::Entity, S); 64]> = SmallVec::default();

        match direction {
            SyncDirection::SpecsToLegion => {
                for (entity, mut component) in (&entities, &mut storage).join() {
                    if let Some(legion_entity) = bimap.read().unwrap().get_by_right(&entity) {
                        let new = if let Some(mut comp) =
                            legion_state.world.get_component_mut::<L>(*legion_entity)
                        {
                            (self.0)(direction, bimap.clone(), Some(component), Some(&mut comp))
                        } else {
                            (self.0)(direction, bimap.clone(), Some(component), None)
                        };

                        if let Some(new_comp) = new.0 {
                            new_specs.push((entity, new_comp));
                        }
                        if let Some(new_comp) = new.1 {
                            new_legion.push((*legion_entity, new_comp));
                        }
                    }
                }
            }
            SyncDirection::LegionToSpecs => {
                use legion::prelude::*;
                let mut query = <(Write<L>)>::query();
                for (entity, mut component) in query.iter_entities(&mut legion_state.world) {
                    if let Some(specs_entity) = bimap.read().unwrap().get_by_left(&entity) {
                        let new = if let Some(specs_component) = storage.get_mut(*specs_entity) {
                            (self.0)(
                                direction,
                                bimap.clone(),
                                Some(specs_component),
                                Some(&mut component),
                            )
                        } else {
                            (self.0)(direction, bimap.clone(), None, Some(&mut component))
                        };

                        if let Some(new_comp) = new.0 {
                            new_specs.push((*specs_entity, new_comp));
                        }
                        if let Some(new_comp) = new.1 {
                            new_legion.push((entity, new_comp));
                        }
                    }
                }
            }
        }

        new_legion
            .drain()
            .for_each(|new| legion_state.world.add_component(new.0, new.1));

        new_specs.drain().for_each(|new| {
            storage.insert(new.0, new.1);
        });
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ResourceSyncer<T>(PhantomData<T>);
impl<T> SyncerTrait for ResourceSyncer<T>
where
    T: legion::resource::Resource,
{
    fn setup(&self, world: &mut specs::World) {
        log::trace!(
            "Creating resource synchronizer for: {:?}",
            std::any::type_name::<T>()
        );
    }

    fn sync(
        &self,
        world: &mut specs::World,
        legion_state: &mut LegionState,
        direction: SyncDirection,
    ) {
        move_resource::<T>(world, legion_state, direction);
    }
}

pub fn move_resource<T: legion::resource::Resource>(
    world: &mut specs::World,
    legion_state: &mut LegionState,
    direction: SyncDirection,
) {
    match direction {
        SyncDirection::SpecsToLegion => {
            if let Some(resource) = world.remove::<T>() {
                legion_state.world.resources.insert(resource);
            }
        }
        SyncDirection::LegionToSpecs => {
            let resource = legion_state.world.resources.remove::<T>();
            if let Some(resource) = resource {
                world.insert(resource);
            }
        }
    }
}

pub fn sync_entities(
    specs_world: &mut specs::World,
    legion_state: &mut LegionState,
    legion_listener_id: ListenerId,
) {
    use smallvec::SmallVec;

    let mut new_entities = SmallVec::<[legion::entity::Entity; 512]>::new();

    let specs_entities = {
        let entity_bimap = legion_state
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
        let new_legion = legion_state
            .world
            .insert((SpecsTag,), specs_entities.clone())
            .iter()
            .enumerate()
            .map(|(i, legion_entity)| {
                log::trace!(
                    "NEW {} - legion:[{:?}] = specs:[{:?}]",
                    i,
                    legion_entity,
                    specs_entities[i].0.specs_entity
                );
                (i, *legion_entity)
            })
            .collect::<Vec<_>>();

        let entity_bimap = legion_state
            .world
            .resources
            .get_mut::<EntitiesBimapRef>()
            .unwrap();
        let mut map = entity_bimap.write().unwrap();
        for (i, entity) in new_legion {
            map.insert(entity, specs_entities[i].0.specs_entity);
        }
    }

    while let Some(event) = legion_state.world.entity_channel().read(legion_listener_id) {
        let entity_bimap = legion_state
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
        let entity_bimap = legion_state
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
