//! Events of the rendering system

use renderer::types::Window;

/// Even to modify a window.
pub struct WindowModifierEvent {
    /// The closure that modifies the window
    pub modify: fn(&mut Window) -> (),
}
