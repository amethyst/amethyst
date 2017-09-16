//! This module contains the `Event` enum and re-exports glutin event
//! types.

pub use winit::{DeviceEvent, DeviceId, ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

/// Events that the engine might send to the user.
#[derive(Clone)]
pub enum Event {
    /// A WindowEvent from the winit crate
    WindowEvent(WindowEvent),
    /// A DeviceEvent from the winit crate
    DeviceEvent(DeviceId, DeviceEvent),
    /// Awakened event from the winit crate
    Awakened,
    // Add more events here as needed.
    #[doc(hidden)]
    /// An unused event, enables _ in match arms.
    __NonExhaustive,
}

/// Added as a resource to the game world with id 0.  Contains all events for a given frame thus
/// far.
pub struct Events(pub Vec<Event>);
