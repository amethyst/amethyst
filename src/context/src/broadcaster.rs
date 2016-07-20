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
//! use amethyst_ecs::{Component, VecStorage};
//!
//! #[derive(Debug)]
//! struct Numeric(u32); 
//! 
//! impl Component for Numeric {
//!     type Storage = VecStorage<Numeric>;
//! }
//!
//! fn main() {
//!     let mut result: Vec<u32> = vec!();
//!     let mut broadcaster = Broadcaster::new();
//!     broadcaster.register::<Numeric>();
//!     for i in 0..10 {
//!         broadcaster.publish().with(Numeric(i)).build();
//!     }
//!     {
//!         let storage = broadcaster.read::<Numeric>();
//!         for entity in broadcaster.poll() {
//!             if let Some(i) = storage.get(entity) {
//!                 result.push(i.0);
//!             }
//!         }
//!     }
//!     assert_eq!(vec!(0,1,2,3,4,5,6,7,8,9), result);
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
