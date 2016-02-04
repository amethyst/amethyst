//! The game world in which entities reside.

use super::dynvec::DynVec;
use super::entity::{Entity, Entities};

use std::any::{Any, TypeId};
use std::collections::HashMap;

type Components = HashMap<TypeId, DynVec>;
type EntityData = HashMap<TypeId, usize>;
// Rebuilder is a function (or a wrapper of that function) that takes an immutable reference to a component T,
// an optional list of immutable references to required components,
// and returns a new component T, that will replace the previous one.
type Rebuilders = HashMap<TypeId, u8>;

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
            let t = TypeId::of::<(Entity, T)>();
            if let Some(c) = self.components.get_mut(&t) {
                c.push((entity, comp));
                return;
            }
            let mut vec = DynVec::new::<(Entity, T)>();
            vec.push((entity, comp));
            self.components.insert(t, vec);
        }
    }

    /// Returns ith component of selected type
    pub fn component<T: Any>(&self, index: usize) -> Option<&(Entity, T)> {
        if let Some(c) = self.components.get(&TypeId::of::<(Entity, T)>()) {
            Some(c.get_component(index))
        } else {
            None
        }
    }

    /// Returns ith mutable component of selected type
    pub fn component_mut<T: Any>(&mut self, index: usize) -> Option<&mut (Entity, T)> {
        if let Some(mut c) = self.components.get_mut(&TypeId::of::<(Entity, T)>()) {
            Some(c.get_component_mut(index))
        } else {
            None
        }
    }
}
