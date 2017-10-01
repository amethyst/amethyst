//! Events of the rendering system

use renderer::types::Window;

/// Event to modify a window. Basically message passing through events.
pub struct WindowModifierEvent {
    /// The closure that modifies the window
    pub modify: Box<Fn(&mut Window) -> () + Send + Sync + 'static>,
}
