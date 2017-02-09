//! World resource that handles all user input.

use fnv::FnvHashMap as HashMap;

use std::collections::hash_map::{Entry, Keys};
use std::iter::Iterator;

use engine::{ElementState, WindowEvent, Event, VirtualKeyCode, MouseButton};

/// Indicates whether a given `VirtualKeyCode` has been queried or not.
#[derive(Eq, PartialEq)]
enum KeyQueryState {
    NotQueried,
    Queried,
}

/// An iterator over the currently pressed down keys.
pub struct PressedKeys<'a> {
    iterator: Keys<'a, VirtualKeyCode, KeyQueryState>,
}

impl<'a> Iterator for PressedKeys<'a> {
    type Item = &'a VirtualKeyCode;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

/// An iterator over the currently pressed down mouse buttons.
pub struct PressedMouseButtons<'a> {
    iterator: Keys<'a, MouseButton, KeyQueryState>,
}

impl<'a> Iterator for PressedMouseButtons<'a> {
    type Item = &'a MouseButton;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

/// This struct holds state information about input devices.
///
/// For example, if a key is pressed on the keyboard, this struct will record
/// that the key is pressed until it is released again. Usage requires pumping input events through
/// this struct, using the following code in the [handle_events] method of [State].
///
/// [State]: ../../../engine/trait.State.html
/// [handle_events]: ../../../engine/trait.State.html#method.handle_events
///
/// ```ignore
/// fn handle_events(&mut self,
///                  events: &[WindowEvent],
///                  _: &mut World,
///                  _: &mut AssetManager,
///                  _: &mut Pipeline)
///                  -> Trans {
///     // ...
///     let mut input_handler = world.write_resource::<InputHandler>();
///     input_handler.update(events);
///     // ...
/// }
/// ```
#[derive(Default)]
pub struct InputHandler {
    pressed_keys: HashMap<VirtualKeyCode, KeyQueryState>,
    pressed_mouse_buttons: HashMap<MouseButton, KeyQueryState>,
}

impl InputHandler {
    /// Creates a new input handler.
    pub fn new() -> InputHandler {
        InputHandler {
            pressed_keys: HashMap::default(),
            pressed_mouse_buttons: HashMap::default(),
        }
    }

    /// Updates the input handler with new engine events.
    pub fn update(&mut self, events: &[WindowEvent]) {
        for event in events {
            match event.payload {
                Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) => {
                    // The check for Entry::Vacant allows more accurate `key_once` calls,
                    // I.e `key_once(key)` is queried after second `Pressed` event.
                    if let Entry::Vacant(entry) = self.pressed_keys.entry(key_code) {
                        entry.insert(KeyQueryState::NotQueried);
                    }
                }
                Event::KeyboardInput(ElementState::Released, _, Some(key_code)) => {
                    self.pressed_keys.remove(&key_code);
                }
                Event::MouseInput(ElementState::Pressed, button) => {
                    if let Entry::Vacant(entry) = self.pressed_mouse_buttons.entry(button) {
                        entry.insert(KeyQueryState::NotQueried);
                    }
                }
                Event::MouseInput(ElementState::Released, button) => {
                    self.pressed_mouse_buttons.remove(&button);
                }
                Event::Focused(false) => {
                    self.pressed_keys.clear();
                    self.pressed_mouse_buttons.clear();
                }
                _ => {}
            }
        }
    }

    /// Returns an iterator for all the pressed down keys.
    pub fn pressed_keys(&self) -> PressedKeys {
        PressedKeys { iterator: self.pressed_keys.keys() }
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
    /// If `key` hasn't been released since the last `key_once()` query, this
    /// function will return false.
    pub fn key_once(&mut self, key: VirtualKeyCode) -> bool {
        if !self.pressed_keys.contains_key(&key) {
            return false;
        }

        if let Some(value) = self.pressed_keys.get_mut(&key) {
            if *value == KeyQueryState::NotQueried {
                *value = KeyQueryState::Queried;
                return true;
            }
        }

        false
    }

    /// Checks if the all the given keys are being pressed and held down.
    ///
    /// If the `keys` haven't been released since the last `key_once()` query,
    /// this function will return false.
    pub fn keys_once(&mut self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().any(|key| self.key_once(*key)) && self.keys_down(keys)
    }

    /// Returns an iterator for all the pressed down mouse buttons.
    pub fn pressed_mouse_buttons(&self) -> PressedMouseButtons {
        PressedMouseButtons { iterator: self.pressed_mouse_buttons.keys() }
    }

    /// Checks if the given mouse button is being pressed.
    pub fn mouse_button_down(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains_key(&button)
    }

    /// Checks if all the given mouse buttons are being pressed.
    pub fn mouse_buttons_down(&self, buttons: &[MouseButton]) -> bool {
        buttons.iter().all(|btn| self.mouse_button_down(*btn))
    }

    /// Checks if the given mouse button is being pressed and held down.
    ///
    /// If `button` hasn't been released since the last `mouse_button_once()` query, this
    /// function will return false.
    pub fn mouse_button_once(&mut self, button: MouseButton) -> bool {
        if !self.pressed_mouse_buttons.contains_key(&button) {
            return false;
        }
        if let Some(value) = self.pressed_mouse_buttons.get_mut(&button) {
            if *value == KeyQueryState::NotQueried {
                *value = KeyQueryState::Queried;
                return true;
            }
        }
        false
    }

    /// Checks if the all the given mouse buttons are being pressed and held down.
    ///
    /// If the `buttons` haven't been released since the last `mouse_button_once()` query,
    /// this function will return false.
    pub fn mouse_buttons_once(&mut self, buttons: &[MouseButton]) -> bool {
        buttons.iter().any(|btn| self.mouse_button_once(*btn)) && self.mouse_buttons_down(buttons)
    }
}
