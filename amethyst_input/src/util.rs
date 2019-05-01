use std::hash::Hash;

use amethyst_core::math::{convert, RealField};
use winit::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::input_handler::InputHandler;

/// If this event was for manipulating a keyboard key then this will return the `VirtualKeyCode`
/// and the new state.
pub fn get_key(event: &Event) -> Option<(VirtualKeyCode, ElementState)> {
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
pub fn is_key_down(event: &Event, key_code: VirtualKeyCode) -> bool {
    if let Some((key, state)) = get_key(event) {
        return key == key_code && state == ElementState::Pressed;
    }

    false
}

/// Returns true if the event passed in is a request to close the game window.
pub fn is_close_requested(event: &Event) -> bool {
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
pub fn get_input_axis_simple<A, B, N: RealField>(name: &Option<A>, input: &InputHandler<A, B>) -> N
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    convert(
        name.as_ref()
            .and_then(|ref n| input.axis_value(n))
            .unwrap_or(0.0),
    )
}
