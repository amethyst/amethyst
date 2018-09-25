use input_handler::InputHandler;
use std::hash::Hash;
use winit::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

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

pub fn is_key_down(event: &Event, key_code: VirtualKeyCode) -> bool {
    let op = get_key(event);
    if let Some((key, state)) = op {
        return key == key_code && state == ElementState::Pressed;
    }
    return false;
}

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
pub fn get_input_axis_simple<A, B>(name: &Option<A>, input: &InputHandler<A, B>) -> f32
where
    A: Send + Sync + Hash + Eq + Clone + 'static,
    B: Send + Sync + Hash + Eq + Clone + 'static,
{
    name.as_ref()
        .and_then(|ref n| input.axis_value(n))
        .unwrap_or(0.0) as f32
}
