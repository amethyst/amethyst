//! This module provides the EntityUuid component, and the EntityUuidMap resource.
//!
//! Together, these can be used to create entity identifiers that can persist across game sessions, and across machines.
//!
//! These properties make them useful for serializing a reference to an entity to disk, such as saving the game, or for
//! identification in a networked multiplayer situation.

use fnv::FnvHashMap;
use specs::{
    Component, Entity, FlaggedStorage,
    System, Write,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// An identifier for an entity which can persist across game sessions, and across machines.
///
/// These properties make it useful for serializing a reference to an entity to disk, such as saving the game, or for
/// identification in a networked multiplayer situation.
///
/// Once the Uuid is initialized in this structure it should not be altered.
#[derive(Debug, Deserialize, Serialize)]
pub struct EntityUuid {
    uuid: Uuid,
    #[serde(skip)]
    dead_signal: Arc<AtomicBool>,
}

impl Component for EntityUuid {
    type Storage = FlaggedStorage<Self>;
}

impl EntityUuid {
    /// Create a new instance with a pre-defined Uuid, typically the Uuid would be deserialized from the disk or network.
    fn new_from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
            dead_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Retrieve the Uuid structure contained in this component.
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl Drop for EntityUuid {
    fn drop(&mut self) {
        self.dead_signal.store(true, Ordering::Relaxed);
    }
}

/// An ECS resource which allows you to create new EntityUuid components, and later retrieve the entities with the contained UUID.
#[derive(Debug, Default)]
pub struct EntityUuidMap {
    map: FnvHashMap<Uuid, (Entity, Arc<AtomicBool>)>,
}

impl EntityUuidMap {
    /// Create an empty instance of this resource.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new EntityUuid component with a new randomly generated Uuid.
    pub fn new_uuid(&mut self, e: Entity) -> EntityUuid {
        self.with_uuid(Uuid::new_v4(), e)
    }

    /// Create a new EntityUuid component with a pre-defined Uuid, typically the Uuid would be deserialized from the disk or network.
    pub fn with_uuid(&mut self, uuid: Uuid, e: Entity) -> EntityUuid {
        let r = EntityUuid::new_from_uuid(uuid);
        self.map.insert(uuid, (e, r.dead_signal.clone()));
        r
    }

    /// Retrieve the entity associated with this Uuid, if any.
    pub fn fetch_entity(&self, uuid: &Uuid) -> Option<Entity> {
        self.map.get(uuid).map(|v| &v.0).cloned()
    }
}

/// This system removes unused Uuid<->Entity mappings from the map.
#[derive(Debug)]
pub struct EntityUuidSystem;

impl<'s> System<'s> for EntityUuidSystem {
    type SystemData = Write<'s, EntityUuidMap>;

    fn run(&mut self, mut map: Self::SystemData) {
        map.map.retain(|_, (_, a)| !a.load(Ordering::Relaxed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{world::Builder, World, DispatcherBuilder};
    use uuid::Uuid;

    #[test]
    fn test_uuid_map() {
        let mut w = World::new();
        w.register::<EntityUuid>();
        let mut d = DispatcherBuilder::new()
            .with(EntityUuidSystem, "entity_uuid", &[])
            .build();
        d.setup(&mut w.res);
        let e1 = w.create_entity().build();
        let e2 = w.create_entity().build();
        let uc1 = w.write_resource::<EntityUuidMap>().new_uuid(e1);
        let u1 = uc1.uuid().clone();
        let uc2 = w.write_resource::<EntityUuidMap>().new_uuid(e2);
        let u2 = uc2.uuid().clone();
        w.write_storage::<EntityUuid>().insert(e1, uc1).unwrap();
        w.write_storage::<EntityUuid>().insert(e2, uc2).unwrap();
        assert_ne!(u1, u2);
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u1), Some(e1));
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u2), Some(e2));
        let u = Uuid::new_v4();
        let e3 = w.create_entity().build();
        let uc3 = w.write_resource::<EntityUuidMap>().with_uuid(u, e3);
        let u3 = uc3.uuid().clone();
        w.write_storage::<EntityUuid>().insert(e3, uc3).unwrap();
        assert_eq!(u3, u);
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u), Some(e3));
        w.delete_entity(e1).unwrap();
        d.dispatch(&mut w.res);
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u1), None);
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u2), Some(e2));
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u3), Some(e3));

        // Now we're going to create an entity with a UUID, delete it, and then
        // make another in the same frame. Since we're only checking by id
        // in the system, this makes sure we don't risk deleting a UUID
        // if the entity index were to be re-used.
        let e4 = w.create_entity().build();
        let uc4 = w.write_resource::<EntityUuidMap>().new_uuid(e4);
        let u4 = uc4.uuid().clone();
        w.write_storage::<EntityUuid>().insert(e4, uc4).unwrap();
        w.delete_entity(e4).unwrap();
        let e5 = w.create_entity().build();
        let uc5 = w.write_resource::<EntityUuidMap>().new_uuid(e5);
        let u5 = uc5.uuid().clone();
        w.write_storage::<EntityUuid>().insert(e5, uc5).unwrap();
        d.dispatch(&mut w.res);
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u1), None);
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u2), Some(e2));
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u3), Some(e3));
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u4), None);
        assert_eq!(w.read_resource::<EntityUuidMap>().fetch_entity(&u5), Some(e5));


    }
}
