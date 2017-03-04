//! World resource that handles all user input.

use std::collections::hash_map::{Entry, HashMap, Keys};
use std::iter::Iterator;

use engine::{ElementState, WindowEvent, Event, VirtualKeyCode};

/// Indicates whether a given `VirtualKeyCode` has been queried or not.
#[derive(Eq, PartialEq)]
enum KeyQueryState {
    NotQueried,
    Queried,
}

/// An iterator over all currently pressed down keys.
pub struct PressedKeysIterator<'a> {
    iterator: Keys<'a, VirtualKeyCode, KeyQueryState>,
}

impl<'a> Iterator for PressedKeysIterator<'a> {
    type Item = &'a VirtualKeyCode;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

/// Processes user input events.
pub struct InputHandler {
    pressed_keys: HashMap<VirtualKeyCode, KeyQueryState>,
}

impl InputHandler {
    /// Creates a new input handler.
    pub fn new() -> InputHandler {
        InputHandler { pressed_keys: HashMap::new() }
    }

    /// Updates the input handler with new engine events.
    pub fn update(&mut self, events: &[WindowEvent]) {
        for event in events {
            match event.payload {
                Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) => {
                    match self.pressed_keys.entry(key_code) {
                        Entry::Occupied(_) => {
                            // nop
                            // Allows more accurate `key_once` calls,
                            // I.e `key_once(key)` is queried after
                            // second `Pressed` event.
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(KeyQueryState::Queried);
                        }
                    }
                }
                Event::KeyboardInput(ElementState::Released, _, Some(key_code)) => {
                    self.pressed_keys.remove(&key_code);
                }
                Event::Focused(false) => self.pressed_keys.clear(),
                _ => {}
            }
        }
    }

    /// Returns an iterator for all the pressed down keys
    pub fn pressed_keys(&self) -> PressedKeysIterator {
        PressedKeysIterator { iterator: self.pressed_keys.keys() }
    }

    /// Checks if the given key is being pressed.
    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.contains_key(&key)
    }

    /// Checks if all the given keys are being pressed.
    pub fn keys_down(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().all(|key| self.key_down(*key))
    }

    /// Checks if the given key is being pressed and held down.
    ///
    /// If `key` hasn't been let go since the last `key_once()` query, this
    /// function will return false.
    pub fn key_once(&mut self, key: VirtualKeyCode) -> bool {
        if !self.pressed_keys.contains_key(&key) {
            return false;
        }

        if let Some(value) = self.pressed_keys.get_mut(&key) {
            // Should be safe
            if *value == KeyQueryState::NotQueried {
                *value = KeyQueryState::Queried;
                return true;
            }
        }

        false
    }

    /// Checks if the all the given keys are being pressed and held down.
    ///
    /// If the `keys` haven't been let go since the last `key_once()` query,
    /// this function will return false.
    pub fn keys_once(&mut self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().any(|key| self.key_once(*key)) && self.keys_down(keys)
    }
}
