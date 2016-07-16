extern crate amethyst_ecs;
extern crate glutin;

use self::amethyst_ecs::{Component, VecStorage};
pub use glutin::{Event, ElementState, ScanCode,
                 VirtualKeyCode, MouseScrollDelta,
                 TouchPhase, MouseButton, Touch};

/// Represents an engine generated event,
/// it can be attached to entities
/// that are published by `Broadcaster`
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
