use winit::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

pub fn get_key(event: &Event) -> Option<VirtualKeyCode> {
    match *event {
        Event::WindowEvent { ref event, .. } => match *event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(ref virtual_keycode),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => Some(*virtual_keycode),
            _ => None,
        },
        _ => None,
    }
}

pub fn is_key(event: &Event, key_code: VirtualKeyCode) -> bool {
    get_key(event) == Some(key_code)
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
