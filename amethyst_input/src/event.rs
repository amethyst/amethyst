use super::local_mouse_button::LocalMouseButton;
use super::local_virtual_key_code::LocalVirtualKeyCode;

use winit::{VirtualKeyCode, MouseButton};

#[derive(Serialize, Deserialize, Clone)]
pub enum InputEvent {
    /// A key was pressed down, sent exactly once per key press.
    KeyPressed {
        #[serde(with = "LocalVirtualKeyCode")]
        key_code: VirtualKeyCode,
        scancode: u32,
    },
    /// A key was released, sent exactly once per key release.
    KeyReleased {
        #[serde(with = "LocalVirtualKeyCode")]
        key_code: VirtualKeyCode,
        scancode: u32,
    },
    /// A unicode character was received by the window.  Good for typing.
    KeyTyped(char),
    /// A mouse button was pressed down, sent exactly once per press.
    MouseButtonPressed(
        #[serde(with = "LocalMouseButton")]
        MouseButton
    ),
    /// A mouse button was released, sent exactly once per release.
    MouseButtonReleased(
        #[serde(with = "LocalMouseButton")]
        MouseButton
    ),
    /// The mouse pointer moved on screen
    MouseMoved { delta_x: f64, delta_y: f64 },
    /// The associated action had one of its keys pressed.
    ActionPressed(String),
    /// The associated action had one of its keys released.
    ActionReleased(String),
}
