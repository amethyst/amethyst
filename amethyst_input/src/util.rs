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
