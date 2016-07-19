//! This module contains `Broadcaster` struct
//! which allows publishing and polling specs
//! entities. It is primarily used for event
//! handling.
//!
//! # Example:
//! ```
//! extern crate amethyst_context;
//! extern crate amethyst_ecs;
//!
//! use amethyst_context::broadcaster::Broadcaster;
//! use amethyst_ecs::{Component, VecStorage}
//!
//! impl Component for i32 {
//!     type Storage = VecStorage;
//! }
//!
//! fn main() {
//!     let broadcaster = Broadcaster::new();
//!     broadcaster.register::<i32>();
//!     for i in 0..10 {
//!         broadcaster.publish::<i32>().with(i).build();
//!     }
//!     {
//!         let storage = broadcaster.read::<i32>();
//!         for entity in broadcaster.poll() {
//!             let i = storage.get(entity);
//!             println!(i);
//!         }
//!     }
//!     broadcaster.clean();
//! }
//! ```

extern crate amethyst_ecs;

use self::amethyst_ecs::{World, Component, EntityBuilder,
                  Storage, Allocator, MaskedStorage, Join, Entity};
use std::sync::RwLockReadGuard;

/// Allows publishing entities
pub struct Broadcaster {
    world: World,
}

impl Broadcaster {
    /// Create new `Broadcaster`
    pub fn new() -> Broadcaster {
        let world = World::new();
        Broadcaster {
            world: world,
        }
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
        self.world.create_now()
    }

    /// Return a vector containing clones of all published entities
    pub fn poll(&self) -> Vec<Entity> {
        let entities = self.world.entities();
        let _entities: Vec<Entity> = entities.iter().map(|e| e.clone()).collect();
        _entities
    }

    /// Access a component storage
    pub fn read<T: Component>(&self) -> Storage<T, RwLockReadGuard<Allocator>, RwLockReadGuard<MaskedStorage<T>>> {
        self.world.read::<T>()
    }

    /// Delete all published entities
    pub fn clean(&mut self) {
        let entities = self.poll();
        for entity in entities {
            self.world.delete_now(entity);
        }
    }
}
