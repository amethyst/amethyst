//! Code for entity lifetime management.

/// An unsigned 64-bit handle to an entity.
pub type Entity = u64;

/// Manages the creation and destruction of entities.
#[derive(Debug)]
pub struct Entities {
    dead: Vec<Entity>,
    next_id: Entity,
}

impl Entities {
    /// Creates a new entity manager.
    pub fn new() -> Entities {
        Entities {
            dead: Vec::new(),
            next_id: 0,
        }
    }

    /// Creates a new entity and returns its handle.
    pub fn create(&mut self) -> Entity {
        if let Some(id) = self.dead.pop() {
            return id;
        }

        let new_entity = self.next_id;
        self.next_id += 1;

        new_entity
    }

    /// Checks whether the given entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.next_id > entity && !self.dead.contains(&entity)
    }

    /// Checks how many entities are currently in the world.
    pub fn num_alive(&self) -> usize {
        self.next_id as usize - self.dead.len()
    }

    /// Destroys the given entity.
    pub fn destroy(&mut self, entity: Entity) {
        if self.is_alive(entity) {
            self.dead.insert(0, entity);
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let mut ents = Entities::new();

        let first = ents.create();
        assert_eq!(first, 0);

        let second = ents.create();
        assert_eq!(second, 1);
    }

    #[test]
    fn recycling() {
        let mut ents = Entities::new();

        let first = ents.create();
        assert_eq!(first, 0);

        let second = ents.create();
        assert_eq!(second, 1);

        let third = ents.create();
        assert_eq!(third, 2);

        ents.destroy(first);
        assert!(!ents.is_alive(first));

        ents.destroy(third);
        assert!(!ents.is_alive(third));

        let rec_first = ents.create();
        assert_eq!(rec_first, 0);

        let rec_second = ents.create();
        assert_eq!(rec_second, 2);
    }
}
