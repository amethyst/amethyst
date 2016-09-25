//! The input handler for the game engine

use std::collections::HashSet;
use event::{ElementState, EngineEvent, Event, VirtualKeyCode};

pub struct InputHandler {
    keys_down: HashSet<VirtualKeyCode>,
}

impl InputHandler {
    /// Create a new InputHandler
    pub fn new() -> InputHandler {
        InputHandler {
            keys_down: HashSet::new(),
        }
    }

    /// Update the input handler with new engine events
    pub fn update(&mut self, events: &[EngineEvent]) {
        for event in events {
            match event.payload {
                Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) => { self.keys_down.insert(key_code); }
                Event::KeyboardInput(ElementState::Released, _, Some(key_code)) => { self.keys_down.remove(&key_code); }
                Event::Focused(false) => self.keys_down.clear(),
                _ => {}
            }
        }
    }

    /// Check if `key` is currently pressed
    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.keys_down.contains(&key)
    }
}