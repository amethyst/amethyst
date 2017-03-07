//! World resource that handles all user input.

use engine::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use fnv::FnvHashMap as HashMap;
use smallvec::SmallVec;
use std::iter::{Chain, Map, Iterator};
use std::slice::Iter;

/// A Button is any kind of digital input that the engine supports.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Button {
    /// Keyboard keys
    Key(VirtualKeyCode),

    /// Mouse buttons
    Mouse(MouseButton),
    //TODO: Add controller buttons here when the engine has support.
}

impl From<VirtualKeyCode> for Button {
    fn from(keycode: VirtualKeyCode) -> Self {
        Button::Key(keycode)
    }
}

impl From<MouseButton> for Button {
    fn from(mouse_button: MouseButton) -> Self {
        Button::Mouse(mouse_button)
    }
}

/// Iterator over keycodes
pub type KeyCodes<'a> = Iter<'a, VirtualKeyCode>;

/// Iterator over MouseButtons
pub type MouseButtons<'a> = Iter<'a, MouseButton>;

/// Represents an axis made up of digital inputs, like W and S or A and D.
/// Two of these could be analogous to a DPAD.
pub struct Axis {
    /// Positive button, when pressed down axis value will return 1 if `neg` is not pressed down.
    pub pos: Button,
    /// Negative button, when pressed down axis value will return -1 if `neg` is not pressed down.
    pub neg: Button,
}

/// An iterator over buttons
pub struct Buttons<'a> {
    iterator: Chain<Map<Iter<'a, MouseButton>, fn(&MouseButton) -> Button>,Map<Iter<'a, VirtualKeyCode>, fn(&VirtualKeyCode) -> Button>>
}

impl<'a> Iterator for Buttons<'a> {
    type Item = Button;

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
    pressed_keys: SmallVec<[VirtualKeyCode; 16]>,
    down_keys: SmallVec<[VirtualKeyCode; 8]>,
    released_keys: SmallVec<[VirtualKeyCode; 8]>,
    pressed_mouse_buttons: SmallVec<[MouseButton; 16]>,
    down_mouse_buttons: SmallVec<[MouseButton; 8]>,
    released_mouse_buttons: SmallVec<[MouseButton; 8]>,
    mouse_position: Option<(i32, i32)>,
    previous_mouse_position: Option<(i32, i32)>,
    axes: HashMap<String, Axis>,
    actions: HashMap<String, SmallVec<[Button; 8]>>,
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
            axes: HashMap::default(),
            actions: HashMap::default(),
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
            match event.payload {
                Event::ReceivedCharacter(c) => {
                    self.text_this_frame.push(c);
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) => {
                    if self.pressed_keys.iter().all(|&k| k != key_code) {
                        self.pressed_keys.push(key_code);
                        self.down_keys.push(key_code);
                    }
                }
                Event::KeyboardInput(ElementState::Released, _, Some(key_code)) => {
                    let index = self.pressed_keys.iter().position(|&k| k == key_code);
                    if let Some(i) = index {
                        self.pressed_keys.swap_remove(i);
                        self.released_keys.push(key_code);
                    }
                }
                Event::MouseInput(ElementState::Pressed, button) => {
                    if self.pressed_mouse_buttons.iter().all(|&b| b != button) {
                        self.pressed_mouse_buttons.push(button);
                        self.down_mouse_buttons.push(button);
                    }
                }
                Event::MouseInput(ElementState::Released, button) => {
                    let index = self.pressed_mouse_buttons.iter().position(|&b| b == button);
                    if let Some(i) = index {
                        self.pressed_mouse_buttons.swap_remove(i);
                        self.released_mouse_buttons.push(button);
                    }
                }
                Event::MouseMoved(x, y) => {
                    self.mouse_position = Some((x, y));
                }
                Event::Focused(false) => {
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

    /// Returns an iterator over pressed down keys.
    pub fn pressed_keys(&self) -> KeyCodes {
        self.pressed_keys.iter()
    }

    /// Returns an iterator over keys pressed on this frame.
    pub fn down_keys(&self) -> KeyCodes {
        self.down_keys.iter()
    }

    /// Returns an iterator over keys released on this frame.
    pub fn released_keys(&self) -> KeyCodes {
        self.released_keys.iter()
    }

    /// Checks if the given key is being pressed.
    pub fn key_is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.iter().any(|&k| k == key)
    }

    /// Checks if the given key was pressed on this frame.
    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.down_keys.iter().any(|&k| k == key)
    }

    /// Checks if a given key was released on this frame.
    pub fn key_released(&self, key: VirtualKeyCode) -> bool {
        self.released_keys.iter().any(|&k| k == key)
    }

    /// Checks if the all the given keys are down and at least one was pressed on this frame.
    pub fn keys_down(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().any(|&key| self.key_down(key)) && self.keys_are_pressed(keys)
    }

    /// Checks if all the given keys are being pressed.
    pub fn keys_are_pressed(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().all(|key| self.key_is_pressed(*key))
    }

    /// Returns an iterator over containing pressed down mouse buttons.
    pub fn pressed_mouse_buttons(&self) -> MouseButtons {
        self.pressed_mouse_buttons.iter()
    }

    /// Returns an iterator over containing mouse buttons pressed on this frame.
    pub fn down_mouse_buttons(&self) -> MouseButtons  {
        self.down_mouse_buttons.iter()
    }

    /// Returns an iterator over mouse buttons released on this frame.
    pub fn released_mouse_buttons(&self) -> MouseButtons {
        self.released_mouse_buttons.iter()
    }

    /// Checks if the given mouse button is being pressed.
    pub fn mouse_button_is_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.iter().any(|&b| b == button)
    }

    /// Checks if the given mouse button was pressed this frame.
    pub fn mouse_button_down(&self, button: MouseButton) -> bool {
        self.down_mouse_buttons.iter().any(|&b| b == button)
    }

    /// Checks if the given mouse button was released this frame.
    pub fn mouse_button_released(&self, button: MouseButton) -> bool {
        self.released_mouse_buttons.iter().any(|&b| b == button)
    }

    /// Checks if the all the given mouse buttons are down and at least one was pressed this frame.
    pub fn mouse_buttons_down(&self, buttons: &[MouseButton]) -> bool {
        buttons.iter().any(|&btn| self.mouse_button_down(btn)) &&
        self.mouse_buttons_are_pressed(buttons)
    }

    /// Gets the current mouse position.
    ///
    /// this method can return None, either if no mouse is connected, or if no mouse events have
    /// been recorded
    pub fn mouse_position(&self) -> Option<(i32, i32)> {
        self.mouse_position
    }

    /// Gets the change in position since the last frame.
    pub fn mouse_position_change(&self) -> (i32, i32) {
        match (self.mouse_position, self.previous_mouse_position) {
            (Some(current), Some(previous)) => (current.0 - previous.0, current.1 - previous.1),
            _ => (0, 0),
        }
    }

    /// Checks if all the given mouse buttons are being pressed.
    pub fn mouse_buttons_are_pressed(&self, buttons: &[MouseButton]) -> bool {
        buttons.iter().all(|btn| self.mouse_button_is_pressed(*btn))
    }

    /// Returns a vector containing the buttons that are currently pressed
    pub fn pressed_buttons(&self) -> Buttons {
        let mouse_buttons = self.pressed_mouse_buttons
            .iter()
            .map((|&mb| Button::Mouse(mb)) as fn(&MouseButton) -> Button);
        let keys = self.pressed_keys.iter().map((|&k| Button::Key(k)) as fn(&VirtualKeyCode) -> Button);
        Buttons {
            iterator: mouse_buttons.chain(keys)
        }
    }

    /// Returns a vector containing the buttons that were pressed this frame
    pub fn down_buttons(&self) -> Buttons {
        let mouse_buttons = self.down_mouse_buttons
            .iter()
            .map((|&mb| Button::Mouse(mb)) as fn(&MouseButton) -> Button);
        let keys = self.down_keys.iter().map((|&k| Button::Key(k)) as fn(&VirtualKeyCode) -> Button);
        Buttons {
            iterator: mouse_buttons.chain(keys)
        }
    }

    /// Returns a vector containing the buttons that were released this frame
    pub fn released_buttons(&self) -> Buttons {
        let mouse_buttons = self.released_mouse_buttons
            .iter()
            .map((|&mb| Button::Mouse(mb)) as fn(&MouseButton) -> Button);
        let keys = self.released_keys.iter().map((|&k| Button::Key(k)) as fn(&VirtualKeyCode) -> Button);
        Buttons {
            iterator: mouse_buttons.chain(keys)
        }
    }

    /// Checks if the given button is currently pressed.
    pub fn button_is_pressed(&self, button: Button) -> bool {
        match button {
            Button::Key(k) => self.key_is_pressed(k),
            Button::Mouse(b) => self.mouse_button_is_pressed(b),
        }
    }

    /// Checks if all the given buttons are pressed.
    pub fn buttons_are_pressed(&self, buttons: &[Button]) -> bool {
        buttons.iter().all(|b| self.button_is_pressed(*b))
    }

    /// Checks if the given button was pressed on this frame.
    pub fn button_down(&self, button: Button) -> bool {
        match button {
            Button::Key(k) => self.key_down(k),
            Button::Mouse(b) => self.mouse_button_down(b),
        }
    }

    /// Checks if the given button was released on this frame.
    pub fn button_released(&self, button: Button) -> bool {
        match button {
            Button::Key(k) => self.key_released(k),
            Button::Mouse(b) => self.mouse_button_released(b),
        }
    }

    /// Checks if the all given buttons are being pressed and at least one was pressed this frame.
    pub fn buttons_down(&self, buttons: &[Button]) -> bool {
        buttons.iter().any(|&b| self.button_down(b)) &&
        buttons.iter().all(|&b| self.button_is_pressed(b))
    }

    /// Returns the value of an axis by the i32 id, if the id doesn't exist this returns None.
    pub fn axis_value<T: AsRef<str>>(&self, id: T) -> Option<f32> {
        self.axes
            .get(id.as_ref())
            .map(|a| {
                let pos = self.button_is_pressed(a.pos);
                let neg = self.button_is_pressed(a.neg);
                if pos == neg {
                    0.0
                } else if pos {
                    1.0
                } else {
                    -1.0
                }
            })
    }

    // Pressed actions has been omitted as the function is a little difficult to write and I
    // can't think of a use case for it.

    /// Checks if the given button is currently pressed.
    pub fn action_is_pressed<T: AsRef<str>>(&self, action: T) -> Option<bool> {
        self.actions
            .get(action.as_ref())
            .map(|ref buttons| buttons.iter().any(|&b| self.button_is_pressed(b)))
    }

    /// Checks if all the given actions are pressed.
    ///
    /// If any action in this list is invalid this will return the id of it in Err.
    pub fn actions_are_pressed<T: AsRef<str>>(&self, actions: &[T]) -> Result<bool, Vec<String>> {
        let mut all_buttons_are_pressed = true;
        let mut bad_values = Vec::new();
        for action in actions {
            if let Some(buttons) = self.actions.get(action.as_ref()) {
                if all_buttons_are_pressed {
                    if !buttons.iter().any(|&b| self.button_is_pressed(b)) {
                        all_buttons_are_pressed = false;
                    }
                }
            } else {
                bad_values.push(action.as_ref().to_string());
            }
        }
        if !bad_values.is_empty() {
            Err(bad_values)
        } else {
            Ok(all_buttons_are_pressed)
        }
    }

    /// Checks if the given action was pressed on this frame.
    pub fn action_down<T: AsRef<str>>(&self, action: T) -> Option<bool> {
        self.actions
            .get(action.as_ref())
            .map(|buttons| buttons.iter().any(|&b| self.button_down(b)))
    }

    /// Checks if the given action was released on this frame.
    pub fn action_released<T: AsRef<str>>(&self, action: T) -> Option<bool> {
        self.actions
            .get(action.as_ref())
            .map(|buttons| buttons.iter().any(|&b| self.button_released(b)))
    }

    /// Checks if the all given actions are being pressed and at least one was pressed this frame.
    ///
    /// If any action in this list is invalid this will return the id of it in Err.
    pub fn actions_down<T: AsRef<str>>(&self, actions: &[T]) -> Result<bool, Vec<String>> {
        let mut all_actions_are_pressed = true;
        let mut any_action_is_pressed_this_frame = false;
        let mut bad_values = Vec::new();
        for action in actions {
            if let Some(buttons) = self.actions.get(action.as_ref()) {
                if !any_action_is_pressed_this_frame {
                    if buttons.iter().any(|&b| self.button_down(b)) {
                        any_action_is_pressed_this_frame = true;
                    }
                }
                if all_actions_are_pressed {
                    if buttons.iter().all(|&b| !self.button_is_pressed(b)) {
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

    /// Assign an axis to an ID value
    ///
    /// This will insert a new axis if no entry for this id exists.
    /// If one does exist this will replace the axis at that id and return it.
    pub fn insert_axis<T: Into<String>>(&mut self, id: T, axis: Axis) -> Option<Axis> {
        self.axes.insert(id.into(), axis)
    }

    /// Removes an axis, this will return the removed axis if successful.
    pub fn remove_axis<T: AsRef<str>>(&mut self, id: T) -> Option<Axis> {
        self.axes.remove(id.as_ref())
    }

    /// Returns a reference to an axis.
    pub fn axis<T: AsRef<str>>(&mut self, id: T) -> Option<&Axis> {
        self.axes.get(id.as_ref())
    }

    /// Gets a list of all axes
    pub fn axes(&self) -> Vec<String> {
        self.axes.keys().map(|k| k.clone()).collect::<Vec<String>>()
    }

    /// Add a button to an action.
    ///
    /// This will insert a new binding between this action and the button.
    pub fn insert_action_binding<T: AsRef<str>>(&mut self, id: T, binding: Button) {
        let mut make_new = false;
        match self.actions.get_mut(id.as_ref()) {
            Some(action_bindings) => {
                if action_bindings.iter().all(|&b| b != binding) {
                    action_bindings.push(binding);
                }
            }
            None => {
                make_new = true;
            }
        }
        if make_new {
            let mut bindings = SmallVec::new();
            bindings.push(binding);
            self.actions.insert(id.as_ref().to_string(), bindings);
        }
    }

    /// Removes an action binding that was assigned previously.
    pub fn remove_action_binding<T: AsRef<str>>(&mut self, id: T, binding: Button) {
        let mut kill_it = false;
        if let Some(action_bindings) = self.actions.get_mut(id.as_ref()) {
            let index = action_bindings.iter().position(|&b| b == binding);
            if let Some(index) = index {
                action_bindings.swap_remove(index);
            }
            kill_it = action_bindings.is_empty();
        }
        if kill_it {
            self.actions.remove(id.as_ref());
        }
    }

    /// Returns an action's bindings.
    pub fn action_bindings<T: AsRef<str>>(&self, id: T) -> Option<&[Button]> {
        self.actions.get(id.as_ref()).map(|a| &**a)
    }

    /// Get's a list of all action bindings
    pub fn actions(&self) -> Vec<String> {
        self.actions.keys().map(|k| k.clone()).collect::<Vec<String>>()
    }
}
