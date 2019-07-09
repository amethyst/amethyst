//! This module provides the EntityUuid component, and the EntityUuidMap resource.
//!
//! Together, these can be used to create entity identifiers that can persist across game sessions, and across machines.
//!
//! These properties make them useful for serializing a reference to an entity to disk, such as saving the game, or for
//! identification in a networked multiplayer situation.

use fnv::FnvHashMap;
use hibitset::BitSetLike;
use shrev::ReaderId;
use specs::{
    storage::ComponentEvent, BitSet, Component, Entities, Entity, FlaggedStorage, ReadStorage,
    Resources, System, SystemData, Write, WriteStorage,
};
use uuid::Uuid;

/// An identifier for an entity which can persist across game sessions, and across machines.
///
/// These properties make it useful for serializing a reference to an entity to disk, such as saving the game, or for
/// identification in a networked multiplayer situation.
///
/// Once the Uuid is initialized in this structure it should not be altered.
#[derive(Debug, Eq, PartialEq)]
pub struct EntityUuid {
    uuid: Uuid,
}

impl Component for EntityUuid {
    type Storage = FlaggedStorage<Self>;
}

impl EntityUuid {
    /// Create a new instance with a new randomly generated Uuid.
    fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    /// Create a new instance with a pre-defined Uuid, typically the Uuid would be deserialized from the disk or network.
    fn new_from_uuid(uuid: Uuid) -> Self {
        Self { uuid }
    }

    /// Retrieve the Uuid structure contained in this component.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

/// An ECS resource which allows you to create new EntityUuid components, and later retrieve the entities with the contained UUID.
#[derive(Debug, Default)]
pub struct EntityUuidMap {
    map: FnvHashMap<Uuid, Entity>,
}

impl EntityUuidMap {
    /// Create an empty instance of this resource.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new EntityUuid component with a new randomly generated Uuid.
    pub fn new_uuid(&mut self, e: Entity) -> EntityUuid {
        let r = EntityUuid::new();
        self.map.insert(r.uuid, e);
        r
    }

    /// Create a new EntityUuid component with a pre-defined Uuid, typically the Uuid would be deserialized from the disk or network.
    pub fn with_uuid(&mut self, uuid: Uuid, e: Entity) -> EntityUuid {
        self.map.insert(uuid, e);
        EntityUuid::new_from_uuid(uuid)
    }

    /// Retrieve the entity associated with this Uuid, if any.
    pub fn fetch_entity(&self, uuid: &Uuid) -> Option<Entity> {
        self.map.get(uuid).cloned()
    }
}

/// This system removes unused Uuid<->Entity mappings from the map.
#[derive(Debug)]
pub struct EntityUuidSystem {
    reader: Option<ReaderId<ComponentEvent>>,
    removed: BitSet,
}

impl EntityUuidSystem {
    /// Create a new instance of this system.
    pub fn new() -> Self {
        Self {
            reader: None,
            removed: BitSet::new(),
        }
    }
}

impl<'s> System<'s> for EntityUuidSystem {
    type SystemData = (
        ReadStorage<'s, EntityUuid>,
        Write<'s, EntityUuidMap>,
        Entities<'s>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.reader = Some(
            res.fetch_mut::<WriteStorage<'_, EntityUuid>>()
                .channel_mut()
                .register_reader(),
        );
    }

    fn run(&mut self, (storage, mut map, entities): Self::SystemData) {
        self.removed.clear();
        let self_events_id = &mut self
            .reader
            .as_mut()
            .expect("EntityUuidSystem::setup was not called before EntityUuidSystem::run");
        for event in storage.channel().read(self_events_id) {
            match event {
                ComponentEvent::Removed(id) => {
                    self.removed.add(*id);
                }
                _ => {}
            }
        }
        for entity in (&self.removed).iter().map(|i| entities.entity(i)) {
            map.map.retain(|_, &mut e| e != entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{world::Builder, World};
    use uuid::Uuid;

    #[test]
    fn test_uuid_map() {
        let mut m = EntityUuidMap::new();
        let mut w = World::new();
        let e1 = w.create_entity().build();
        let e2 = w.create_entity().build();
        let u1 = m.new_uuid(e1);
        let u2 = m.new_uuid(e2);
        assert_ne!(u1, u2);
        assert_eq!(m.fetch_entity(u1.uuid()), Some(e1));
        assert_eq!(m.fetch_entity(u2.uuid()), Some(e2));
        let u = Uuid::new_v4();
        let e3 = w.create_entity().build();
        assert_eq!(m.with_uuid(u, e3).uuid(), &u);
        assert_eq!(m.fetch_entity(&u), Some(e3));
    }
}
