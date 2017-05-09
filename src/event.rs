//! This module contains the `WindowEvent` type and re-exports glutin event
//! types.

pub use winit::{ElementState, ModifiersState, MouseButton, MouseScrollDelta,
                ScanCode, Touch, TouchPhase, VirtualKeyCode as Key, WindowEvent};

use winit::Event as WinitEvent;

/// Generic engine event.
#[derive(Debug)]
pub enum Event {
    /// An asset event.
    Asset(String),
    /// A window event.
    Window(WindowEvent),
    /// User-defined event.
    User(String),
}

impl From<WinitEvent> for Event {
    fn from(e: WinitEvent) -> Event {
        let WinitEvent::WindowEvent { event, .. } = e;
        Event::Window(event)
    }
}

impl From<WindowEvent> for Event {
    fn from(e: WindowEvent) -> Event {
        Event::Window(e)
    }
}

/// Iterable stream of events.
#[derive(Debug, Default)]
pub struct EventsIter(Vec<Event>);

impl Iterator for EventsIter {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        self.0.pop()
    }
}

impl<E: Into<Event>> From<Vec<E>> for EventsIter {
    fn from(e: Vec<E>) -> EventsIter {
        EventsIter(e.into_iter().map(|e| e.into()).collect())
    }
}
