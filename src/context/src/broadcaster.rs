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
//! use amethyst_ecs::{Component, VecStorage, Join};
//!
//! struct UserEvent {
//!     pub data: i32,
//! }
//!
//! impl Component for UserEvent {
//!     type Storage = VecStorage<UserEvent>;
//! }
//!
//! fn main() {
//!     let mut broadcaster = Broadcaster::new();
//!     broadcaster.register::<UserEvent>();
//!     for i in 0..10 {
//!         let user_component = UserEvent { data: i };
//!         broadcaster.publish().with::<UserEvent>(user_component).build();
//!     }
//!     {
//!         let user_events = broadcaster.read::<UserEvent>();
//!         for user_event in user_events.iter() {
//!             println!("{0}", user_event.data);
//!         }
//!     }
//!     broadcaster.clean();
//! }
//! ```

extern crate amethyst_ecs;

use self::amethyst_ecs::{World, Component, EntityBuilder,
                  Storage, Allocator, MaskedStorage, Join};
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

    /// Access a component storage
    pub fn read<T: Component>(&self) -> Storage<T, RwLockReadGuard<Allocator>, RwLockReadGuard<MaskedStorage<T>>> {
        self.world.read::<T>()
    }

    /// Delete all published entities
    pub fn clean(&mut self) {
        for entity in self.world.entities().iter() {
            self.world.delete_later(entity);
        }
        self.world.maintain();
    }
}
