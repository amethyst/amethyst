//! The game world in which entities reside.

use super::dynvec::DynVec;
use super::entity::{Entity, Entities};

use std::any::{Any, TypeId};
use std::collections::{HashMap, BTreeMap};

type Components = HashMap<TypeId, DynVec>;
type EntityData = HashMap<TypeId, usize>;

/// A collection of entities and their respective components.
#[derive(Debug)]
pub struct World {
    components: Components,
    entities: Entities,
    ent_data: BTreeMap<Entity, EntityData>
}

impl World {
    /// Creates a new empty world.
    pub fn new() -> World {
        World {
            components: Components::new(),
            entities: Entities::new(),
            ent_data: BTreeMap::new()
        }
    }

    /// Creates a new entity in the world and returns a handle to it.
    pub fn create_entity(&mut self) -> Entity {
        let id = self.entities.create();
        self.ent_data.insert(id, EntityData::new());
        id
    }

    /// Destroys a given entity and removes its components.
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entities.destroy(entity);
        //TODO: remove components
        self.ent_data.remove(&entity);
    }

    /// Attaches a component to an entity.
    pub fn insert_component<T: Any>(&mut self, entity: Entity, comp: T) -> Option<usize> {
		let ent_data: &mut EntityData = match self.ent_data.get_mut(&entity) {
			Some(s) => s,
			None => return None
		};
        let t = TypeId::of::<(Entity, T)>();
        //is_alive check may be redundant, as we already check availability of EntityData.
        if self.entities.is_alive(entity) && !ent_data.contains_key(&t) {
            if let Some(c) = self.components.get_mut(&t) {
				let id = c.add((entity, comp));
				ent_data.insert(t, id);
                return Some(id);
            }
			let mut vec = DynVec::new::<(Entity, T)>();
			vec.add((entity, comp));
			self.components.insert(t, vec);
			ent_data.insert(t, 0);
			Some(0)
        } else {
			None
		}
    }

	pub fn remove_component<T: Any>(&mut self, entity: Entity) {
        let t = TypeId::of::<(Entity, T)>();
        let id = self.ent_data[&entity][&t];
		if let Some(c) = self.components.get_mut(&t) {
			c.remove::<T>(id);
		}
	}

    /// Returns ith component of selected type
    pub fn component<T: Any>(&self, index: usize) -> Option<&(Entity, T)> {
        if let Some(c) = self.components.get(&TypeId::of::<(Entity, T)>()) {
            c.get_component(index)
        } else {
            None
        }
    }

    /// Returns ith mutable component of selected type
    pub fn component_mut<T: Any>(&mut self, index: usize) -> Option<&mut (Entity, T)> {
        if let Some(mut c) = self.components.get_mut(&TypeId::of::<(Entity, T)>()) {
            c.get_component_mut(index)
        } else {
            None
        }
    }
}
