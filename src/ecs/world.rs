//! The game world in which entities reside.

use super::component::Component;
use super::entity::{Entity, Entities};

use std::any::Any;

/// A collection of entities and their respective components.
#[derive(Debug)]
pub struct World {
    entities: Entities,
    components: Vec<Component>,
}

impl World {
    /// Creates a new empty world.
    pub fn new() -> World {
        World {
            components: Vec::new(),
            entities: Entities::new(),
        }
    }

    /// Creates a new entity in the world and returns a handle to it.
    pub fn create_entity(&mut self) -> Entity {
        self.entities.create()
    }

    /// Destroys a given entity and deallocates its components.
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entities.destroy(entity);
        self.components.retain(|c| (*c).owner != entity);
    }

    /// Attaches a component to an entity.
    pub fn insert_component<T: Any>(&mut self, entity: Entity, comp: T) {
        if self.entities.is_alive(entity) {
            self.components.push(Component::new(entity, comp));
            self.components.sort_by(|next, prev| next.owner.cmp(&prev.owner));
        }
    }

    /// Gets an immutable reference to all of the components in the world.
    pub fn get_components(&self) -> &Vec<Component> {
        &self.components
    }

    /// Gets a mutable reference to all of the components in the world.
    pub fn get_components_mut(&mut self) -> &mut Vec<Component> {
        &mut self.components
    }
}
