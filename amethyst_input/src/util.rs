use winit::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

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

pub fn get_mouse_button(event: &Event) -> Option<(MouseButton, ElementState)> {
    match *event {
        Event::WindowEvent { ref event, .. } => match *event {
            WindowEvent::MouseInput {
                ref button, state, ..
            } => Some((*button, state)),
            _ => None,
        },
        _ => None,
    }
}

/// Returns true if the specified mouse button is down this frame.
/// Does not know about previous states.
///
/// Buttons: Left, Right, Middle, Other(u8)
pub fn is_mouse_button_down(event: &Event, mouse_button: MouseButton) -> bool {
    let op = get_mouse_button(event);
    if let Some((key, state)) = op {
        return key == mouse_button && state == ElementState::Pressed;
    }
    return false;
}

/// Returns true if the specified mouse button was released this frame.
/// Does not know about previous states.
///
/// Buttons: Left, Right, Middle, Other(u8)
pub fn was_mouse_button_released(event: &Event, mouse_button: MouseButton) -> bool {
    let op = get_mouse_button(event);
    if let Some((key, state)) = op {
        return key == mouse_button && state == ElementState::Released;
    }
    return false;
}
