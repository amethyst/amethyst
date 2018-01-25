use winit::{MouseButton, VirtualKeyCode};

use super::local_mouse_button::LocalMouseButton;
use super::local_virtual_key_code::LocalVirtualKeyCode;

/// A Button is any kind of digital input that the engine supports.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Button {
    /// Virtual Keyboard keys, use this when the letter on the key matters
    /// more than the position of the key.
    Key(#[serde(with = "LocalVirtualKeyCode")]
    VirtualKeyCode),

    /// Scan code from keyboard, use this when the position of the key matters
    /// more than letter on the key.
    ScanCode(u32),

    /// Mouse buttons
    Mouse(
        #[serde(with = "LocalMouseButton")]
        MouseButton,
    ),
    //TODO: Add controller buttons here when the engine has support.
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
