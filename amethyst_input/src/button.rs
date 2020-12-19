use serde::{Deserialize, Serialize};
use winit::event::{MouseButton, VirtualKeyCode};

use super::{controller::ControllerButton, scroll_direction::ScrollDirection};

/// A Button is any kind of digital input that the engine supports.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum Button {
    /// Virtual Keyboard keys, use this when the letter on the key matters
    /// more than the position of the key.
    Key(VirtualKeyCode),

    /// Scan code from keyboard, use this when the position of the key matters
    /// more than the letter on the key.
    ScanCode(u32),

    /// Mouse buttons
    Mouse(MouseButton),

    /// Mouse wheel (Do not use these with an emulated axis, instead use the MouseWheel axis.)
    MouseWheel(ScrollDirection),

    /// Controller buttons matching SDL controller model.
    /// A tuple of sequential controller_id in order of connection
    /// and specific type of used controller button.
    Controller(u32, ControllerButton),
}

impl From<VirtualKeyCode> for Button {
    fn from(keycode: VirtualKeyCode) -> Self {
        Button::Key(keycode)
    }
}

impl From<MouseButton> for Button {
    fn from(mouse_button: MouseButton) -> Self {
        Button::Mouse(mouse_button)
    }
}
