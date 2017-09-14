//! This module contains the `Event` enum and re-exports glutin event
//! types.

pub use winit::{ElementState, Event as WinitEvent, KeyboardInput, MouseButton, VirtualKeyCode,
                WindowEvent};

/// Events that the engine might send to the user.
pub enum Event {
    /// An event from the Winit crate
    WinitEvent(WinitEvent),
    /// Unused event, enables _ in match statements
    UnusedEvent,
    // Add more events here as needed.
}

/// Added as a resource to the game world with id 0.  Contains all events for a given frame thus
/// far.
pub struct Events(pub Vec<Event>);
