//! World resource that handles all user input.

use fnv::FnvHashMap as HashMap;

use std::iter::{Iterator, Chain, Map};
use std::slice::Iter;

use engine::{ElementState, WindowEvent, Event, VirtualKeyCode, MouseButton};

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

/// An iterator over the currently pressed down keys.
pub struct PressedKeys<'a> {
    iterator: Iter<'a, VirtualKeyCode>,
}

impl<'a> Iterator for PressedKeys<'a> {
    type Item = &'a VirtualKeyCode;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

/// An iterator over the currently pressed down mouse buttons.
pub struct PressedMouseButtons<'a> {
    iterator: Iter<'a, MouseButton>,
}

impl<'a> Iterator for PressedMouseButtons<'a> {
    type Item = &'a MouseButton;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

/// Iterator over currently pressed down keys and mouse buttons.
pub struct PressedButtons<'a> {
    iterator: Chain<Map<PressedMouseButtons<'a>, fn(&MouseButton) -> Button>,
                    Map<PressedKeys<'a>, fn(&VirtualKeyCode) -> Button>>,
}

impl<'a> Iterator for PressedButtons<'a> {
    type Item = Button;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

/// Represents an axis made up of digital inputs, like W and S or A and D.
/// Two of these could be analogous to a DPAD.
pub struct Axis {
    /// Positive button, when pressed down the axis value will return 1 if `neg` is not pressed down.
    pub pos: Button,
    /// Negative button, when pressed down the axis value will return -1 if `neg` is not pressed down.
    pub neg: Button,
}

/// Represents a single frame of input data from the mouse and keyboard.
#[derive(Default)]
struct FrameInputData {
    pressed_keys: Vec<VirtualKeyCode>,
    pressed_mouse_buttons: Vec<MouseButton>,
    mouse_position: Option<(i32, i32)>,
}

impl FrameInputData {
    pub fn new() -> FrameInputData {
        FrameInputData {
            pressed_keys: Vec::new(),
            pressed_mouse_buttons: Vec::new(),
            mouse_position: None,
        }
    }

    /// Pushes data from this frame into the previous frame parameter.
    fn advance_frame(&mut self, previous_frame: &mut FrameInputData) {
        previous_frame.mouse_position = self.mouse_position;
        previous_frame.pressed_keys.clear();
        previous_frame
            .pressed_keys
            .extend_from_slice(&self.pressed_keys);
        previous_frame.pressed_mouse_buttons.clear();
        previous_frame
            .pressed_mouse_buttons
            .extend_from_slice(&self.pressed_mouse_buttons);
    }

    /// Returns an iterator for all the pressed down keys.
    fn pressed_keys(&self) -> PressedKeys {
        PressedKeys { iterator: self.pressed_keys.iter() }
    }

    /// Checks if the given key is being pressed.
    fn key_is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.iter().any(|&k| k == key)
    }

    /// Checks if all the given keys are being pressed.
    fn keys_are_pressed(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().all(|key| self.key_is_pressed(*key))
    }

    /// Returns an iterator for all the pressed down mouse buttons.
    fn pressed_mouse_buttons(&self) -> PressedMouseButtons {
        PressedMouseButtons { iterator: self.pressed_mouse_buttons.iter() }
    }

    /// Checks if the given mouse button is being pressed.
    fn mouse_button_is_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.iter().any(|&b| b == button)
    }

    /// Checks if all the given mouse buttons are being pressed.
    fn mouse_buttons_are_pressed(&self, buttons: &[MouseButton]) -> bool {
        buttons.iter().all(|btn| self.mouse_button_is_pressed(*btn))
    }

    /// Returns an iterator over the buttons that are currently pressed
    fn pressed_buttons(&self) -> PressedButtons {
        let mouse_button_convert = mouse_button_to_button as fn(&MouseButton) -> Button;
        let key_convert = key_to_button as fn(&VirtualKeyCode) -> Button;
        let mouse_buttons = self.pressed_mouse_buttons().map(mouse_button_convert);
        let keys = self.pressed_keys().map(key_convert);
        PressedButtons { iterator: mouse_buttons.chain(keys) }
    }

    /// Checks if the given button is currently pressed.
    fn button_is_pressed(&self, button: Button) -> bool {
        match button {
            Button::Key(k) => self.key_is_pressed(k),
            Button::Mouse(b) => self.mouse_button_is_pressed(b),
        }
    }

    /// Checks if all the given buttons are pressed.
    fn buttons_are_pressed(&self, buttons: &[Button]) -> bool {
        buttons.iter().all(|b| self.button_is_pressed(*b))
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
    current_frame: FrameInputData,
    previous_frame: FrameInputData,
    axes: HashMap<i32, Axis>,
    actions: HashMap<i32, Button>,
    text_this_frame: String,
}

impl InputHandler {
    /// Creates a new input handler.
    pub fn new() -> InputHandler {
        InputHandler {
            current_frame: FrameInputData::new(),
            previous_frame: FrameInputData::new(),
            axes: HashMap::default(),
            actions: HashMap::default(),
            text_this_frame: String::new(),
        }
    }

    /// Updates the input handler with new engine events.
    pub fn update(&mut self, events: &[WindowEvent]) {
        // Before processing these events store the input states of the previous frame.
        self.current_frame.advance_frame(&mut self.previous_frame);
        self.text_this_frame.clear();
        for event in events {
            match event.payload {
                Event::ReceivedCharacter(c) => {
                    self.text_this_frame.push(c);
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(key_code)) => {
                    if self.current_frame
                           .pressed_keys
                           .iter()
                           .all(|&k| k != key_code) {
                        self.current_frame.pressed_keys.push(key_code);
                    }
                }
                Event::KeyboardInput(ElementState::Released, _, Some(key_code)) => {
                    let index = self.current_frame
                        .pressed_keys
                        .iter()
                        .position(|&k| k == key_code);
                    if let Some(i) = index {
                        self.current_frame.pressed_keys.swap_remove(i);
                    }
                }
                Event::MouseInput(ElementState::Pressed, button) => {
                    if self.current_frame
                           .pressed_mouse_buttons
                           .iter()
                           .all(|&b| b != button) {
                        self.current_frame.pressed_mouse_buttons.push(button);
                    }
                }
                Event::MouseInput(ElementState::Released, button) => {
                    let index = self.current_frame
                        .pressed_mouse_buttons
                        .iter()
                        .position(|&b| b == button);
                    if let Some(i) = index {
                        self.current_frame.pressed_mouse_buttons.swap_remove(i);
                    }
                }
                Event::MouseMoved(x, y) => {
                    self.current_frame.mouse_position = Some((x, y));
                }
                Event::Focused(false) => {
                    self.current_frame.pressed_keys.clear();
                    self.current_frame.pressed_mouse_buttons.clear();
                    self.current_frame.mouse_position = None;
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

    /// Returns an iterator for all the pressed down keys.
    pub fn pressed_keys(&self) -> PressedKeys {
        self.current_frame.pressed_keys()
    }

    /// Checks if the given key is being pressed.
    pub fn key_is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.current_frame.key_is_pressed(key)
    }

    /// Checks if all the given keys are being pressed.
    pub fn keys_are_pressed(&self, keys: &[VirtualKeyCode]) -> bool {
        self.current_frame.keys_are_pressed(keys)
    }

    /// Checks if the given key was pressed on this frame.
    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.current_frame.key_is_pressed(key) && !self.previous_frame.key_is_pressed(key)
    }

    /// Checks if a given key was released on this frame.
    pub fn key_released(&self, key: VirtualKeyCode) -> bool {
        !self.current_frame.key_is_pressed(key) && self.previous_frame.key_is_pressed(key)
    }

    /// Checks if the all the given keys are down and at least one was pressed on this frame.
    pub fn keys_down(&self, keys: &[VirtualKeyCode]) -> bool {
        keys.iter().any(|&key| self.key_down(key)) && self.keys_are_pressed(keys)
    }

    /// Returns an iterator for all the pressed down mouse buttons.
    pub fn pressed_mouse_buttons(&self) -> PressedMouseButtons {
        self.current_frame.pressed_mouse_buttons()
    }

    /// Checks if the given mouse button is being pressed.
    pub fn mouse_button_is_pressed(&self, button: MouseButton) -> bool {
        self.current_frame.mouse_button_is_pressed(button)
    }

    /// Checks if all the given mouse buttons are being pressed.
    pub fn mouse_buttons_are_pressed(&self, buttons: &[MouseButton]) -> bool {
        self.current_frame.mouse_buttons_are_pressed(buttons)
    }

    /// Checks if the given mouse button was pressed this frame.
    pub fn mouse_button_down(&self, button: MouseButton) -> bool {
        self.current_frame.mouse_button_is_pressed(button) &&
        !self.previous_frame.mouse_button_is_pressed(button)
    }

    /// Checks if the given mouse button was released this frame.
    pub fn mouse_button_released(&self, button: MouseButton) -> bool {
        !self.current_frame.mouse_button_is_pressed(button) &&
        self.previous_frame.mouse_button_is_pressed(button)
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
        self.current_frame.mouse_position
    }

    /// Gets the change in position since the last frame.
    pub fn mouse_position_change(&self) -> (i32, i32) {
        match (self.current_frame.mouse_position, self.previous_frame.mouse_position) {
            (Some(current), Some(previous)) => (current.0 - previous.0, current.1 - previous.1),
            _ => (0, 0),
        }
    }

    /// Returns an iterator over the buttons that are currently pressed
    pub fn pressed_buttons(&self) -> PressedButtons {
        self.current_frame.pressed_buttons()
    }

    /// Checks if the given button is currently pressed.
    pub fn button_is_pressed(&self, button: Button) -> bool {
        self.current_frame.button_is_pressed(button)
    }

    /// Checks if all the given buttons are pressed.
    pub fn buttons_are_pressed(&self, buttons: &[Button]) -> bool {
        self.current_frame.buttons_are_pressed(buttons)
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

    /// Checks if the all the given buttons are being pressed and at least one was pressed this frame.
    pub fn buttons_down(&self, buttons: &[Button]) -> bool {
        buttons.iter().any(|&b| self.button_down(b)) &&
        buttons.iter().all(|&b| self.button_is_pressed(b))
    }

    /// Returns the value of an axis by the i32 id, if the id doesn't exist this returns None.
    pub fn axis_value(&self, id: i32) -> Option<f32> {
        self.axes
            .get(&id)
            .map(|a| {
                let pos = self.button_down(a.pos);
                let neg = self.button_down(a.neg);
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
    pub fn action_is_pressed(&self, action: i32) -> Option<bool> {
        self.actions
            .get(&action)
            .map(|&b| self.button_is_pressed(b))
    }

    /// Checks if all the given actions are pressed.
    ///
    /// If any action in this list is invalid this will return the id of it in Err.
    pub fn actions_are_pressed(&self, actions: &[i32]) -> Result<bool, Vec<i32>> {
        let mut all_buttons_are_pressed = true;
        let mut bad_values = Vec::new();
        for action in actions {
            if let Some(button) = self.actions.get(action) {
                if all_buttons_are_pressed && !self.button_is_pressed(*button) {
                    all_buttons_are_pressed = false;
                }
            } else {
                bad_values.push(*action);
            }
        }
        if !bad_values.is_empty() {
            Err(bad_values)
        } else {
            Ok(all_buttons_are_pressed)
        }
    }

    /// Checks if the given action was pressed on this frame.
    pub fn action_down(&self, action: i32) -> Option<bool> {
        self.actions.get(&action).map(|&b| self.button_down(b))
    }

    /// Checks if the given action was released on this frame.
    pub fn action_released(&self, action: i32) -> Option<bool> {
        self.actions.get(&action).map(|&b| self.button_released(b))
    }

    /// Checks if the all the given actions are being pressed and at least one was pressed this frame.
    ///
    /// If any action in this list is invalid this will return the id of it in Err.
    pub fn actions_down(&self, actions: &[i32]) -> Result<bool, Vec<i32>> {
        let mut all_buttons_are_pressed = true;
        let mut any_button_is_pressed_this_frame = false;
        let mut bad_values = Vec::new();
        for action in actions {
            if let Some(button) = self.actions.get(action) {
                if !any_button_is_pressed_this_frame && self.button_down(*button) {
                    any_button_is_pressed_this_frame = true;
                }
                if all_buttons_are_pressed && !self.button_is_pressed(*button) {
                    all_buttons_are_pressed = false;
                }
            } else {
                bad_values.push(*action);
            }
        }
        if !bad_values.is_empty() {
            Err(bad_values)
        } else {
            Ok(all_buttons_are_pressed && any_button_is_pressed_this_frame)
        }
    }

    /// Assign an axis to an ID value
    ///
    /// This will insert a new axis if no entry for this id exists.
    /// If one does exist this will replace the axis at that id and return it.
    pub fn insert_axis(&mut self, id: i32, axis: Axis) -> Option<Axis> {
        self.axes.insert(id, axis)
    }

    /// Removes an axis, this will return the removed axis if successful.
    pub fn remove_axis(&mut self, id: i32) -> Option<Axis> {
        self.axes.remove(&id)
    }

    /// Returns a reference to an axis.
    pub fn get_axis(&mut self, id: i32) -> Option<&Axis> {
        self.axes.get(&id)
    }

    /// Assign an action to an ID value
    ///
    /// This will insert a new action if no entry for this id exists.
    /// If one does exist this will replace the action at that id and return it.
    pub fn insert_action(&mut self, id: i32, binding: Button) -> Option<Button> {
        self.actions.insert(id, binding)
    }

    /// Removes an action, this will return the removed action if successful.
    pub fn remove_action(&mut self, id: i32) -> Option<Button> {
        self.actions.remove(&id)
    }

    /// Returns a reference to an action's button.
    pub fn get_action(&mut self, id: i32) -> Option<&Button> {
        self.actions.get(&id)
    }
}

fn mouse_button_to_button(mb: &MouseButton) -> Button {
    Button::Mouse(*mb)
}

fn key_to_button(k: &VirtualKeyCode) -> Button {
    Button::Key(*k)
}
