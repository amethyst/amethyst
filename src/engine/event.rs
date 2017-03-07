//! This module contains the `WindowEvent` type and re-exports glutin event
//! types.

use ecs::{Component, VecStorage};

pub use glutin::{Event, ElementState, ScanCode, VirtualKeyCode, MouseScrollDelta, TouchPhase,
                 MouseButton, Touch};
use std::ops::{Deref, DerefMut};

/// A window-generated event.
pub struct WindowEvent {
    /// Underlying Glutin event type.
    pub payload: Event,
}

impl WindowEvent {
    /// Creates a new window event from the given Glutin event.
    pub fn new(event: Event) -> WindowEvent {
        WindowEvent { payload: event }
    }
}

impl Component for WindowEvent {
    type Storage = VecStorage<WindowEvent>;
}

impl Deref for WindowEvent {
    type Target = Event;

    fn deref(&self) -> &Event {
        &self.payload
    }
}

impl DerefMut for WindowEvent {
    fn deref_mut(&mut self) -> &mut Event {
        &mut self.payload
    }
}
