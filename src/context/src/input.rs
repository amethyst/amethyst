//! The input handler for the game engine

use std::collections::HashSet;
use event::{ElementState, EngineEvent, Event, VirtualKeyCode, MouseButton };

pub struct InputHandler {
    keys_down: HashSet<VirtualKeyCode>,
    buttons_down: HashSet<MouseButton>,
    pointer_location: (i32,i32)
}

impl InputHandler {
    /// Create a new InputHandler
    pub fn new() -> InputHandler {
        InputHandler {
            keys_down: HashSet::new(),
            buttons_down: HashSet::new(),
            pointer_location: (0,0)
        }
    }

    /// Update the input handler with new engine events
    pub fn update(&mut self, events: &[EngineEvent]) {
        for event in events {
            match event.payload {
                Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) => { self.keys_down.insert(key_code); }
                Event::KeyboardInput(ElementState::Released, _, Some(key_code)) => { self.keys_down.remove(&key_code); }
                Event::Focused(false) => self.keys_down.clear(),
                Event::MouseMoved(x,y) => { self.pointer_location = (x,y) ; },
                Event::MouseInput(ElementState::Pressed,button) => { self.buttons_down.insert(button); },
                Event::MouseInput(ElementState::Released, button) => { self.buttons_down.remove(&button); },
                _ => {}
            }
        }
    }

    /// Check if `key` is currently pressed
    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    ///Check if `button` is pressed.
    pub fn button_down(&self, button: MouseButton) -> bool {
        self.buttons_down.contains(&button)
    }

    ///Get pointer location.
    pub fn pointer_location(&self) -> (i32, i32) {
        self.pointer_location
    }
}
