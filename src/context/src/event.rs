//! This module contains the `EngineEvent` component and reexports glutin event types.

extern crate amethyst_ecs;
extern crate glutin;

use self::amethyst_ecs::{Component, VecStorage};
pub use self::glutin::{Event, ElementState, ScanCode,
                       VirtualKeyCode, MouseScrollDelta,
                       TouchPhase, MouseButton, Touch};

/// Represents an engine generated event,
/// it can be attached to entities
/// that are published by `Broadcaster`.
/// Currently it is just a wraper around
/// `glutin::Event`.
pub struct EngineEvent {
    pub payload: Event,
}

impl EngineEvent {
    /// Create an EnginEvent from a glutin::Event
    pub fn new(event: Event) -> EngineEvent {
        EngineEvent {
            payload: event,
        }
    }
}

impl Component for EngineEvent {
    type Storage = VecStorage<EngineEvent>;
}
