use crate::{input_handler::InputHandler, BindingTypes};
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

/// If this event was for manipulating a keyboard key then this will return the `VirtualKeyCode`
/// and the new state.
pub fn get_key(event: &Event<()>) -> Option<(VirtualKeyCode, ElementState)> {
    match *event {
        Event::WindowEvent { ref event, .. } => match *event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(ref virtual_keycode),
                        state,
                        ..
                    },
                ..
            } => Some((*virtual_keycode, state)),
            _ => None,
        },
        _ => None,
    }
}

/// Returns true if the event passed in is a key down event for the
/// provided `VirtualKeyCode`.
pub fn is_key_down(event: &Event<()>, key_code: VirtualKeyCode) -> bool {
    if let Some((key, state)) = get_key(event) {
        return key == key_code && state == ElementState::Pressed;
    }

    false
}

/// Returns true if the event passed in is a key up event for the
/// provided `VirtualKeyCode`.
pub fn is_key_up(event: &Event<()>, key_code: VirtualKeyCode) -> bool {
    if let Some((key, state)) = get_key(event) {
        return key == key_code && state == ElementState::Released;
    }

    false
}

/// Returns true if the event passed in is a request to close the game window.
pub fn is_close_requested(event: &Event<()>) -> bool {
    match *event {
        Event::WindowEvent { ref event, .. } => match *event {
            WindowEvent::CloseRequested => true,
            _ => false,
        },
        _ => false,
    }
}

/// Gets the input axis value from the `InputHandler`.
/// If the name is None, it will return the default value of the axis (0.0).
pub fn get_input_axis_simple<T: BindingTypes>(
    name: &Option<T::Axis>,
    input: &InputHandler<T>,
) -> f32 {
    name.as_ref()
        .and_then(|ref n| input.axis_value(n))
        .unwrap_or(0.0)
}

/// If this event was for manipulating a mouse button, this will return the `MouseButton`
/// and the new state.
pub fn get_mouse_button(event: &Event<()>) -> Option<(MouseButton, ElementState)> {
    match *event {
        Event::WindowEvent { ref event, .. } => match *event {
            WindowEvent::MouseInput { button, state, .. } => Some((button, state)),
            _ => None,
        },
        _ => None,
    }
}

/// Returns true if the event passed in is a mouse button down event for the
/// provided `MouseButton`.
pub fn is_mouse_button_down(event: &Event<()>, button: MouseButton) -> bool {
    if let Some((pressed_button, state)) = get_mouse_button(event) {
        pressed_button == button && state == ElementState::Pressed
    } else {
        false
    }
}
