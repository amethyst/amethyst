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
//!         for user_comp in user_comps.iter() {
//!             println!("{0}", user_comp.data);
//!         }
//!     }
//!     bc.clean();
//! }
//! ```

use std::sync::RwLockReadGuard;

use ecs::{World, Component, EntityBuilder, Storage, Allocator, MaskedStorage, Join};

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
        self.world.create_now()
    }

    /// Access a component storage
    pub fn read<T: Component>
        (&self)
         -> Storage<T, RwLockReadGuard<Allocator>, RwLockReadGuard<MaskedStorage<T>>> {
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
