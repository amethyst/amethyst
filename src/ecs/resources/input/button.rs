use event::{VirtualKeyCode, MouseButton};

use super::local_mouse_button::LocalMouseButton;
use super::local_virtual_key_code::LocalVirtualKeyCode;

/// A Button is any kind of digital input that the engine supports.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Button {
    /// Keyboard keys
    Key(#[serde(with = "LocalVirtualKeyCode")]
        VirtualKeyCode),

    /// Mouse buttons
    Mouse(#[serde(with = "LocalMouseButton")]
          MouseButton),
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

/// Describes an input state for a button.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum ButtonState {
    /// Button is pressed
    Pressed(ChangeState),
    /// Button is released
    Released(ChangeState),
}

/// Indicates when the ButtonState it is contained within changed
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum ChangeState {
    /// Button was either pressed or released this frame.
    ThisFrame,
    /// Button was either pressed or released in any frame.
    Currently,
}