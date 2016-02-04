//! The game world in which entities reside.

use super::anyvec::AnyVec;
use super::entity::{Entity, Entities};

use std::any::Any;
use std::collections::BTreeMap;

type Components = BTreeMap<Entity, AnyVec>;

/// A collection of entities and their respective components.
#[derive(Debug)]
pub struct World {
    components: Components,
    entities: Entities,
}

impl World {
    /// Creates a new empty world.
    pub fn new() -> World {
        World {
            components: Components::new(),
            entities: Entities::new(),
        }
    }

    /// Creates a new entity in the world and returns a handle to it.
    pub fn create_entity(&mut self) -> Entity {
        let id = self.entities.create();
        self.components.insert(id.clone(), AnyVec::new());
        id
    }

    /// Destroys a given entity and deallocates its components.
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.components.remove(&entity);
        self.entities.destroy(entity);
    }

    /// Attaches a component to an entity.
    pub fn insert_component<T: Any>(&mut self, entity: Entity, comp: T) {
        if self.entities.is_alive(entity) {
            if let Some(c) = self.components.get_mut(&entity) {
                c.push(comp);
            }
        }
    }
}

