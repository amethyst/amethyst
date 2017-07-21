//! This module contains `Broadcaster` struct which allows publishing and
//! polling specs entities. It is primarily used for event handling.
//!
//! # Example:
//! ```
//! extern crate amethyst;
//!
//! use amethyst::ecs::{Component, VecStorage, Join};
//! use amethyst::ecs::resources::Broadcaster;
//!
//! struct UserComponent {
//!     pub data: i32,
//! }
//!
//! impl Component for UserComponent {
//!     type Storage = VecStorage<UserComponent>;
//! }
//!
//! fn main() {
//!     let mut bc = Broadcaster::new();
//!     bc.register::<UserComponent>();
//!     for i in 0..10 {
//!         let user_comp = UserComponent { data: i };
//!         bc.publish().with::<UserComponent>(user_comp).build();
//!     }
//!     {
//!         let user_comps = bc.read::<UserComponent>();
//!         for user_comp in user_comps.join() {
//!             println!("{0}", user_comp.data);
//!         }
//!     }
//!     bc.clean();
//! }
//! ```

use ecs::{Component, EntityBuilder, Join, ReadStorage, World};

/// Allows publishing entities
pub struct Broadcaster {
    world: World,
}

impl Broadcaster {
    /// Create new `Broadcaster`
    pub fn new() -> Broadcaster {
        let world = World::new();
        Broadcaster { world: world }
    }

    /// Add a custom `Component` with which
    /// entities can be built and published
    /// using `Broadcaster::publish()`
    pub fn register<T: Component>(&mut self) {
        self.world.register::<T>();
    }

    /// Build and publish an entity,
    /// using `EntityBuilder` syntax
    pub fn publish(&mut self) -> EntityBuilder {
        self.world.create_entity()
    }

    /// Access a component storage
    pub fn read<T: Component>(&self) -> ReadStorage<T> {
        self.world.read::<T>()
    }

    /// Delete all published entities
    pub fn clean(&mut self) {
        let entities = {
            self.world.entities().join().collect::<Vec<_>>()
        };
        for entity in entities {
            self.world.delete_entity(entity);
        }
    }
}
