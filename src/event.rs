//! This module contains the `WindowEvent` type and re-exports glutin event
//! types.

pub use winit::{DeviceEvent, WindowEvent};

use winit::Event as WinitEvent;

/// Generic engine event.
#[derive(Clone, Debug)]
pub enum Event {
    /// An asset event.
    Asset(String),
    /// Event loop awakened event.
    Awakened,
    /// A device event.
    Device(DeviceEvent),
    /// A window event.
    Window(WindowEvent),
    /// User-defined event.
    User(String),
}

impl From<WinitEvent> for Event {
    fn from(e: WinitEvent) -> Event {
        match e {
            WinitEvent::Awakened => Event::Awakened,
            WinitEvent::DeviceEvent { event, .. } => Event::Device(event),
            WinitEvent::WindowEvent { event, .. } => Event::Window(event),
        }
    }
}

impl From<DeviceEvent> for Event {
    fn from(e: DeviceEvent) -> Event {
        Event::Device(e)
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
