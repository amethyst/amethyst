use amethyst::renderer::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

pub fn is_exit(event: Event) -> bool {
    match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => true,
            _ => false,
        },
        _ => false,
    }
}

pub fn is_pause(event: Event) -> bool {
    match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => true,
            _ => false,
        },
        _ => false,
    }
}
