//! World resource that handles all user input.

use event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};
use smallvec::SmallVec;
use super::*;

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
    /// Maps inputs to actions and axes.
    pub bindings: Bindings,
    pressed_keys: SmallVec<[VirtualKeyCode; 16]>,
    down_keys: SmallVec<[VirtualKeyCode; 16]>,
    released_keys: SmallVec<[VirtualKeyCode; 16]>,
    pressed_mouse_buttons: SmallVec<[MouseButton; 16]>,
    down_mouse_buttons: SmallVec<[MouseButton; 16]>,
    released_mouse_buttons: SmallVec<[MouseButton; 16]>,
    mouse_position: Option<(f64, f64)>,
    previous_mouse_position: Option<(f64, f64)>,
    text_this_frame: String,
}

impl InputHandler {
    /// Creates a new input handler.
    pub fn new() -> InputHandler {
        InputHandler {
            pressed_keys: SmallVec::new(),
            down_keys: SmallVec::new(),
            released_keys: SmallVec::new(),
            pressed_mouse_buttons: SmallVec::new(),
            down_mouse_buttons: SmallVec::new(),
            released_mouse_buttons: SmallVec::new(),
            mouse_position: None,
            previous_mouse_position: None,
            bindings: Bindings::default(),
            text_this_frame: String::new(),
        }
    }

    /// Updates the input handler with new engine events.
    pub fn update(&mut self, events: &[WindowEvent]) {
        // Before processing these events clear the single frame vectors
        self.down_keys.clear();
        self.released_keys.clear();
        self.down_mouse_buttons.clear();
        self.released_mouse_buttons.clear();
        self.previous_mouse_position = self.mouse_position;
        self.text_this_frame.clear();
        for event in events {
            match *event {
                WindowEvent::ReceivedCharacter(c) => {
                    self.text_this_frame.push(c);
                }
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(key_code),
                        ..
                    },
                    ..
                } => {
                    if self.pressed_keys.iter().all(|&k| k != key_code) {
                        self.pressed_keys.push(key_code);
                        self.down_keys.push(key_code);
                    }
                }
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(key_code),
                        ..
                    },
                    ..
                } => {
                    let index = self.pressed_keys.iter().position(|&k| k == key_code);
                    if let Some(i) = index {
                        self.pressed_keys.swap_remove(i);
                        self.released_keys.push(key_code);
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button,
                    ..
                } => {
                    if self.pressed_mouse_buttons.iter().all(|&b| b != button) {
                        self.pressed_mouse_buttons.push(button);
                        self.down_mouse_buttons.push(button);
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button,
                    ..
                } => {
                    let index = self.pressed_mouse_buttons
                        .iter()
                        .position(|&b| b == button);
                    if let Some(i) = index {
                        self.pressed_mouse_buttons.swap_remove(i);
                        self.released_mouse_buttons.push(button);
                    }
                }
                WindowEvent::MouseMoved {
                    position: (x, y),
                    ..
                } => {
                    self.mouse_position = Some((x, y));
                }
                WindowEvent::Focused(false) => {
                    self.pressed_keys.clear();
                    self.pressed_mouse_buttons.clear();
                    self.mouse_position = None;
                }
                _ => {}
            }
        }
    }

    /// Returns a string representation of all text entered this frame.
    ///
    /// Intended for use with text entry fields, insert this string at the cursor position
    /// every frame.
    pub fn text_entered(&self) -> &str {
        self.text_this_frame.as_str()
    }

    /// Returns an iterator over all keys in the given state, does not support Released(Currently).
    pub fn keys_that_are(&self, state: ButtonState) -> KeyCodes {
        if let Pressed(ThisFrame) = state {
            self.down_keys.iter()
        }
        else if let Pressed(Currently) = state {
            self.pressed_keys.iter()
        }
        else if let Released(ThisFrame) = state {
            self.released_keys.iter()
        }
        else {
            panic!("Released(Currently) is not supported in this context.");
        }
    }

    /// Checks if a key matches the description given by state.
    pub fn key_is(&self, key: VirtualKeyCode, state: ButtonState) -> bool {
        match state {
            Pressed(ThisFrame) => {
                self.down_keys.iter().any(|&k| k == key)
            }
            Pressed(Currently) => {
                self.pressed_keys.iter().any(|&k| k == key)
            }
            Released(ThisFrame) => {
                self.released_keys.iter().any(|&k| k == key)
            }
            Released(Currently) => {
                self.pressed_keys.iter().all(|&k| k != key)
            }
        }
    }

    /// Checks if the all the given keys are down and at least one was pressed on this frame.
    pub fn keys_down(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().any(|&key| self.key_is(key, Pressed(ThisFrame))) &&
        keys.iter().all(|&key| self.key_is(key, Pressed(Currently)))
    }

    /// Returns an iterator over all mouse buttons in the given state, does not support Released(Currently).
    pub fn mouse_buttons_that_are(&self, state: ButtonState) -> MouseButtons {
        if let Pressed(ThisFrame) = state {
            return self.down_mouse_buttons.iter();
        }
        else if let Pressed(Currently) = state {
            return self.pressed_mouse_buttons.iter();
        }
        else if let Released(ThisFrame) = state {
            return self.released_mouse_buttons.iter();
        }
        panic!("Released(Currently) is not supported in this context.");
    }

    /// Checks if a mouse button matches the description given by state.
    pub fn mouse_button_is(&self, mouse_button: MouseButton, state: ButtonState) -> bool {
        match state {
            Pressed(ThisFrame) => {
                self.down_mouse_buttons.iter().any(|&mb| mb == mouse_button)
            }
            Pressed(Currently) => {
                self.pressed_mouse_buttons.iter().any(|&mb| mb == mouse_button)
            }
            Released(ThisFrame) => {
                self.released_mouse_buttons.iter().any(|&mb| mb == mouse_button)
            }
            Released(Currently) => {
                self.pressed_mouse_buttons.iter().all(|&mb| mb != mouse_button)
            }
        }
    }

    /// Checks if the all the given mouse buttons are down and at least one was pressed this frame.
    pub fn mouse_buttons_down(&self, buttons: &[MouseButton]) -> bool {
        buttons.iter().any(|&btn| self.mouse_button_is(btn, Pressed(ThisFrame))) &&
        buttons.iter().all(|&btn| self.mouse_button_is(btn, Pressed(Currently)))
    }

    /// Gets the current mouse position.
    ///
    /// this method can return None, either if no mouse is connected, or if no mouse events have
    /// been recorded
    pub fn mouse_position(&self) -> Option<(f64, f64)> {
        self.mouse_position
    }

    /// Gets the change in position since the last frame.
    pub fn mouse_position_change(&self) -> (f64, f64) {
        match (self.mouse_position, self.previous_mouse_position) {
            (Some(current), Some(previous)) => (current.0 - previous.0, current.1 - previous.1),
            _ => (0f64, 0f64),
        }
    }

    /// Returns an iterator over all buttons in the given state, does not support Released(Currently).
    pub fn buttons_that_are(&self, state: ButtonState) -> Buttons {
        let mouse_buttons;
        let keys;
        match state {
            Pressed(ThisFrame) => {
                mouse_buttons = &self.down_mouse_buttons;
                keys = &self.down_keys;
            }
            Pressed(Currently) => {
                mouse_buttons = &self.pressed_mouse_buttons;
                keys = &self.pressed_keys;
            }
            Released(ThisFrame) => {
                mouse_buttons = &self.released_mouse_buttons;
                keys = &self.released_keys;
            }
            Released(Currently) => {
                panic!("Released(Currently) is not supported in this context.");
            }
        }
        let mouse_buttons = mouse_buttons
            .iter()
            .map((|&mb| Button::Mouse(mb)) as fn(&MouseButton) -> Button);
        let keys = keys
            .iter()
            .map((|&k| Button::Key(k)) as fn(&VirtualKeyCode) -> Button);
        Buttons { iterator: mouse_buttons.chain(keys) }
    }

    /// Checks if a button matches the description given by state.
    pub fn button_is(&self, button: Button, state: ButtonState) -> bool {
        match button {
            Button::Key(k) => self.key_is(k, state),
            Button::Mouse(b) => self.mouse_button_is(b, state),
        }
    }

    /// Checks if the all given buttons are being pressed and at least one was pressed this frame.
    pub fn buttons_down(&self, buttons: &[Button]) -> bool {
        buttons.iter().any(|&b| self.button_is(b, Pressed(ThisFrame))) &&
        buttons.iter().all(|&b| self.button_is(b, Pressed(Currently)))
    }

    /// Returns the value of an axis by the string id, if the id doesn't exist this returns None.
    pub fn axis_value<T: AsRef<str>>(&self, id: T) -> Option<f64> {
        self.bindings
            .axes
            .get(id.as_ref())
            .map(|a| {
                let pos = self.button_is(a.pos, Pressed(Currently));
                let neg = self.button_is(a.neg, Pressed(Currently));
                if pos == neg {
                    0.0
                } else if pos {
                    1.0
                } else {
                    -1.0
                }
            })
    }

    /// Returns true if any of the action is in the given state.  Returns None if
    /// an invalid action was provided.
    pub fn action_is<T: AsRef<str>>(&self, action: T, state: ButtonState) -> Option<bool> {
        match state {
            Released(Currently) => {
                self.bindings
                    .actions
                    .get(action.as_ref())
                    .map(|ref buttons| buttons.iter().all(|&b| self.button_is(b, state)))
            }
            _ => {
                self.bindings
                    .actions
                    .get(action.as_ref())
                    .map(|ref buttons| buttons.iter().any(|&b| self.button_is(b, state)))
            }
        }

    }

    /// Checks if the all given actions are being pressed and at least one was pressed this frame.
    ///
    /// If any action in this list is invalid this will return the id of it in Err.
    pub fn actions_down<T: AsRef<str>>(&self, actions: &[T]) -> Result<bool, Vec<String>> {
        let mut all_actions_are_pressed = true;
        let mut any_action_is_pressed_this_frame = false;
        let mut bad_values = Vec::new();
        for action in actions {
            if let Some(buttons) = self.bindings.actions.get(action.as_ref()) {
                if !any_action_is_pressed_this_frame {
                    if buttons.iter().any(|&b| self.button_is(b, Pressed(ThisFrame))) {
                        any_action_is_pressed_this_frame = true;
                    }
                }
                if all_actions_are_pressed {
                    if buttons.iter().all(|&b| self.button_is(b, Released(Currently))) {
                        all_actions_are_pressed = false;
                    }
                }
            } else {
                bad_values.push(action.as_ref().to_string());
            }
        }
        if !bad_values.is_empty() {
            Err(bad_values)
        } else {
            Ok(all_actions_are_pressed && any_action_is_pressed_this_frame)
        }
    }
}
