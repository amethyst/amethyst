extern crate glutin;
extern crate amethyst_ecs;

pub use glutin::{Event, ElementState, ScanCode,
                 VirtualKeyCode, MouseScrollDelta,
                 TouchPhase, MouseButton, Touch};

use self::amethyst_ecs::{World, Component, VecStorage, EntityBuilder,
                  Storage, Allocator, MaskedStorage, Join, Entity};
use std::sync::RwLockReadGuard;

pub struct EngineEvent {
    pub payload: Event,
}

impl EngineEvent {
    pub fn new(event: Event) -> EngineEvent {
        EngineEvent {
            payload: event,
        }
    }
}

impl Component for EngineEvent {
    type Storage = VecStorage<EngineEvent>;
}

pub struct Broadcaster {
    world: World,
}

impl Broadcaster {
    pub fn new() -> Broadcaster {
        let world = World::new();
        let mut broadcaster = Broadcaster {
            world: world,
        };
        broadcaster.register::<EngineEvent>();
        broadcaster
    }

    pub fn register<T: Component>(&mut self) {
        self.world.register::<T>();
    }

    pub fn publish(&mut self) -> EntityBuilder {
        self.world.create_now()
    }

    pub fn poll(&self) -> Vec<Entity> {
        let entities = self.world.entities();
        let _entities: Vec<Entity> = entities.iter().map(|e| e.clone()).collect();
        _entities
    }

    pub fn read<T: Component>(&self) -> Storage<T, RwLockReadGuard<Allocator>, RwLockReadGuard<MaskedStorage<T>>> {
        self.world.read::<T>()
    }

    pub fn clean(&mut self) {
        let entities = self.poll();
        for entity in entities {
            self.world.delete_now(entity);
        }
    }
}
