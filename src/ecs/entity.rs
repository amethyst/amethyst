//! Code for entity lifetime management.

/// An unsigned 64-bit handle to an entity.
pub type Entity = u64;

/// Manages the creation and destruction of entities.
#[derive(Debug)]
pub struct Entities {
    alive: Vec<Entity>,
    dead: Vec<Entity>,
    next_id: Entity,
}

impl Entities {
    /// Creates a new entity manager.
    pub fn new() -> Entities {
        Entities {
            alive: Vec::new(),
            dead: Vec::new(),
            next_id: 0,
        }
    }

    /// Creates a new entity and returns its handle.
    pub fn create(&mut self) -> Entity {
        if let Some(id) = self.dead.pop() {
            self.alive.push(id.clone());
            return id;
        }

        let new_entity = self.next_id;
        self.alive.push(new_entity.clone());
        self.next_id += 1;

        new_entity
    }

    /// Checks whether the given entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive.iter().chain(&self.dead).filter(|&e| *e == entity).count() == 1
    }

    /// Destroys the given entity.
    pub fn destroy(&mut self, entity: Entity) {
        if self.is_alive(entity) {
            self.alive.retain(|&e| e != entity);
            self.dead.push(entity);
        }
    }
}
