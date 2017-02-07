//! This module contains the `EngineEvent` component and reexports glutin event types.

extern crate specs;
extern crate glutin;

use self::specs::{Component, VecStorage};
pub use self::glutin::{Event, ElementState, ScanCode, VirtualKeyCode, MouseScrollDelta, TouchPhase, MouseButton, Touch};

/// Represents a window generated event,
/// it can be attached to an entity published by `Broadcaster`.
/// Currently it is just a wraper around
/// `glutin::Event`.
pub struct WindowEvent {
    pub payload: Event,
}

impl WindowEvent {
    /// Create an EnginEvent from a glutin::Event
    pub fn new(event: Event) -> WindowEvent {
        WindowEvent { payload: event }
    }
}

impl Component for WindowEvent {
    type Storage = VecStorage<WindowEvent>;
}
