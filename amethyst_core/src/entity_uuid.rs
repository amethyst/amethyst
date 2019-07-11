//! This module provides the EntityUuid component, and the EntityUuidMap resource.
//!
//! Together, these can be used to create entity identifiers that can persist across game sessions, and across machines.
//!
//! These properties make them useful for serializing a reference to an entity to disk, such as saving the game, or for
//! identification in a networked multiplayer situation.

use fnv::FnvHashMap;
use specs::{Entity, Entities, System, Write};
use uuid::Uuid;

/// An ECS resource that presents a bi-directional mapping between Uuids and Entities.
#[derive(Debug, Default)]
pub struct EntityUuidMap {
    uuid_to_entity: FnvHashMap<Uuid, Entity>,
    entity_to_uuid: FnvHashMap<Entity, Uuid>,
}

impl EntityUuidMap {
    /// Create an empty instance of this resource.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Uuid and associate it with the entity given.
    pub fn new_uuid(&mut self, e: Entity) -> Uuid {
        let u = Uuid::new_v4();
        self.add_relationship(u, e);
        u
    }

    /// Create a new relationship between an Entity and a Uuid.
    pub fn add_relationship(&mut self, uuid: Uuid, e: Entity) {
        self.uuid_to_entity.insert(uuid, e);
        self.entity_to_uuid.insert(e, uuid);
    }

    /// Retrieve the entity associated with this Uuid, if any.
    pub fn fetch_entity(&self, uuid: &Uuid) -> Option<Entity> {
        self.uuid_to_entity.get(uuid).cloned()
    }

    /// Retrieve the Uuid associated with this entity, if any.
    pub fn fetch_uuid(&self, entity: Entity) -> Option<&Uuid> {
        self.entity_to_uuid.get(&entity)
    }

    /// Remove the relationship containing this Uuid. Returns true if
    /// successful.
    pub fn remove_by_uuid(&mut self, uuid: &Uuid) -> bool {
        if let Some(e) = self.fetch_entity(uuid) {
            self.entity_to_uuid.remove(&e);
            self.uuid_to_entity.remove(uuid);
            return true;
        }
        false
    }

    /// Remove the relationship containing this Entity. Returns true if
    /// successful.
    pub fn remove_by_entity(&mut self, e: Entity) -> bool {
        if let Some(u) = self.fetch_uuid(e).cloned() {
            self.entity_to_uuid.remove(&e);
            self.uuid_to_entity.remove(&u);
            return true;
        }
        false
    }

    /// Clear out old mappings that are no longer useful.
    pub fn maintain(&mut self, entities: &Entities<'_>) {
        self.entity_to_uuid.retain(|e, _u| entities.is_alive(*e));
        self.uuid_to_entity.retain(|_u, e| entities.is_alive(*e));
    }
}

/// This system removes unused Uuid<->Entity mappings from the map.
#[derive(Debug)]
pub struct EntityUuidSystem;

impl<'s> System<'s> for EntityUuidSystem {
    type SystemData = (
        Write<'s, EntityUuidMap>,
        Entities<'s>,
    );

    fn run(&mut self, (mut map, entities): Self::SystemData) {
        map.maintain(&entities);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, world::Builder, System, SystemData};

    #[test]
    fn test_uuid_map() {
        let mut w = World::new();
        w.add_resource(EntityUuidMap::new());
        let e1 = w.create_entity().build();
        let e2 = w.create_entity().build();
        let u3;
        let u4;
        {
            let mut e = w.write_resource::<EntityUuidMap>();
            let u1 = e.new_uuid(e1);
            let u2 = e.new_uuid(e2);
            assert_eq!(e.fetch_entity(&u1), Some(e1));
            assert_eq!(e.fetch_entity(&u2), Some(e2));
            assert_eq!(e.fetch_uuid(e1), Some(&u1));
            assert_eq!(e.fetch_uuid(e2), Some(&u2));
            assert!(e.remove_by_entity(e1));
            assert_eq!(e.fetch_entity(&u1), None);
            assert_eq!(e.fetch_entity(&u2), Some(e2));
            assert_eq!(e.fetch_uuid(e1), None);
            assert_eq!(e.fetch_uuid(e2), Some(&u2));
            assert!(e.remove_by_uuid(&u2));
            assert_eq!(e.fetch_entity(&u1), None);
            assert_eq!(e.fetch_entity(&u2), None);
            assert_eq!(e.fetch_uuid(e1), None);
            assert_eq!(e.fetch_uuid(e2), None);
            u3 = Uuid::new_v4();
            u4 = Uuid::new_v4();
            e.add_relationship(u3, e1);
            e.add_relationship(u4, e2);
            assert_eq!(e.fetch_entity(&u3), Some(e1));
            assert_eq!(e.fetch_entity(&u4), Some(e2));
            assert_eq!(e.fetch_uuid(e1), Some(&u3));
            assert_eq!(e.fetch_uuid(e2), Some(&u4));
        }
        
        w.delete_entity(e1).unwrap();
        w.delete_entity(e2).unwrap();
        let mut s = EntityUuidSystem;
        s.run(<EntityUuidSystem as System<'_>>::SystemData::fetch(&w.res));
        let e = w.write_resource::<EntityUuidMap>();
        assert_eq!(e.fetch_entity(&u3), None);
        assert_eq!(e.fetch_entity(&u4), None);
        assert_eq!(e.fetch_uuid(e1), None);
        assert_eq!(e.fetch_uuid(e2), None);
    }
}
