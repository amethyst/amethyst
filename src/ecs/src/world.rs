//! The game world in which entities reside.

use super::dynvec::DynVec;
use super::entity::{Entity, Entities};

use std::any::{Any,TypeId};
use std::collections::HashMap;

type Components = HashMap<TypeId, DynVec>;

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
        id
    }

    /// Destroys a given entity and deallocates its components.
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entities.destroy(entity);
    }

    /// Attaches a component to an entity.
    pub fn insert_component<T: Any>(&mut self, entity: Entity, comp: T) {
        if self.entities.is_alive(entity) {
			let t = TypeId::of::<T>();
            if let Some(c) = self.components.get_mut(&t) {
                c.push(comp);
                return;
            }
            let mut vec = DynVec::new::<T>();
            vec.push(comp);
            self.components.insert(t, vec);
        }
    }
    
    /// Returns ith component of selected type
    pub fn get_component<T: 'static + Any>(&self, index: usize) -> Option<&T> {
		if let Some(c) = self.components.get(&TypeId::of::<T>()) {
			Some(c.get_component(index))
		} else {
			None
		}
	}
}

