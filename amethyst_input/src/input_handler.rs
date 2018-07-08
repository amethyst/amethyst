//! World resource that handles all user input.

use std::borrow::Borrow;
use std::hash::Hash;

use amethyst_core::shrev::EventChannel;
use smallvec::SmallVec;
use winit::{DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode,
            WindowEvent};

use super::event::InputEvent;
use super::event::InputEvent::*;
use super::*;

/// This struct holds state information about input devices.
///
/// For example, if a key is pressed on the keyboard, this struct will record
/// that the key is pressed until it is released again.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct InputHandler<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    /// Maps inputs to actions and axes.
    pub bindings: Bindings<AX, AC>,
    /// Encodes the VirtualKeyCode and corresponding scancode.
    pressed_keys: SmallVec<[(VirtualKeyCode, u32); 12]>,
    pressed_mouse_buttons: SmallVec<[MouseButton; 12]>,
    mouse_position: Option<(f64, f64)>,
}

impl<AX, AC> InputHandler<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    /// Creates a new input handler.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<AX, AC> InputHandler<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    /// Updates the input handler with a new engine event.
    ///
    /// The Amethyst game engine will automatically call this if the InputHandler is attached to
    /// the world as a resource with id 0.
    pub fn send_event(&mut self, event: &Event, event_handler: &mut EventChannel<InputEvent<AC>>) {
        match *event {
            Event::WindowEvent { ref event, .. } => match *event {
                WindowEvent::ReceivedCharacter(c) => {
                    event_handler.single_write(KeyTyped(c));
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(key_code),
                            scancode,
                            ..
                        },
                    ..
                } => if self.pressed_keys.iter().all(|&k| k.0 != key_code) {
                    self.pressed_keys.push((key_code, scancode));
                    event_handler.iter_write(
                        [
                            KeyPressed { key_code, scancode },
                            ButtonPressed(Button::Key(key_code)),
                            ButtonPressed(Button::ScanCode(scancode)),
                        ].iter()
                            .cloned(),
                    );
                    for (k, v) in self.bindings.actions.iter() {
                        for &button in v {
                            if Button::Key(key_code) == button {
                                event_handler.single_write(ActionPressed(k.clone()));
                            }
                            if Button::ScanCode(scancode) == button {
                                event_handler.single_write(ActionPressed(k.clone()));
                            }
                        }
                    }
                },
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(key_code),
                            scancode,
                            ..
                        },
                    ..
                } => {
                    let index = self.pressed_keys.iter().position(|&k| k.0 == key_code);
                    if let Some(i) = index {
                        self.pressed_keys.swap_remove(i);
                        event_handler.iter_write(
                            [
                                KeyReleased { key_code, scancode },
                                ButtonReleased(Button::Key(key_code)),
                                ButtonReleased(Button::ScanCode(scancode)),
                            ].iter()
                                .cloned(),
                        );
                        for (k, v) in self.bindings.actions.iter() {
                            for &button in v {
                                if Button::Key(key_code) == button {
                                    event_handler.single_write(ActionReleased(k.clone()));
                                }
                                if Button::ScanCode(scancode) == button {
                                    event_handler.single_write(ActionReleased(k.clone()));
                                }
                            }
                        }
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button,
                    ..
                } => {
                    let mouse_button = button;
                    if self.pressed_mouse_buttons
                        .iter()
                        .all(|&b| b != mouse_button)
                    {
                        self.pressed_mouse_buttons.push(mouse_button);
                        event_handler.iter_write(
                            [
                                MouseButtonPressed(mouse_button),
                                ButtonPressed(Button::Mouse(mouse_button)),
                            ].iter()
                                .cloned(),
                        );
                        for (k, v) in self.bindings.actions.iter() {
                            for &button in v {
                                if Button::Mouse(mouse_button) == button {
                                    event_handler.single_write(ActionPressed(k.clone()));
                                }
                            }
                        }
                    }
                }
                WindowEvent::MouseInput {
                    state: ElementState::Released,
                    button,
                    ..
                } => {
                    let mouse_button = button;
                    let index = self.pressed_mouse_buttons
                        .iter()
                        .position(|&b| b == mouse_button);
                    if let Some(i) = index {
                        self.pressed_mouse_buttons.swap_remove(i);
                        event_handler.iter_write(
                            [
                                MouseButtonReleased(mouse_button),
                                ButtonReleased(Button::Mouse(mouse_button)),
                            ].iter()
                                .cloned(),
                        );
                        for (k, v) in self.bindings.actions.iter() {
                            for &button in v {
                                if Button::Mouse(mouse_button) == button {
                                    event_handler.single_write(ActionReleased(k.clone()));
                                }
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved {
                    position: (x, y), ..
                } => {
                    if let Some((old_x, old_y)) = self.mouse_position {
                        event_handler.single_write(CursorMoved {
                            delta_x: x - old_x,
                            delta_y: y - old_y,
                        });
                    }
                    self.mouse_position = Some((x, y));
                }
                WindowEvent::Focused(false) => {
                    self.pressed_keys.clear();
                    self.pressed_mouse_buttons.clear();
                    self.mouse_position = None;
                }
                _ => {}
            },
            Event::DeviceEvent { ref event, .. } => match *event {
                DeviceEvent::MouseMotion {
                    delta: (delta_x, delta_y),
                } => {
                    event_handler.single_write(MouseMoved { delta_x, delta_y });
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Returns an iterator over all keys that are down.
    pub fn keys_that_are_down(&self) -> impl Iterator<Item=VirtualKeyCode> + '_ {
        self.pressed_keys
            .iter()
            .map((|k| k.0) as fn(&(VirtualKeyCode, u32)) -> VirtualKeyCode)
    }

    /// Checks if a key is down.
    pub fn key_is_down(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.iter().any(|&k| k.0 == key)
    }

    /// Returns an iterator over all pressed mouse buttons
    pub fn mouse_buttons_that_are_down(&self) -> impl Iterator<Item=&MouseButton> {
        self.pressed_mouse_buttons.iter()
    }

    /// Checks if a mouse button is down.
    pub fn mouse_button_is_down(&self, mouse_button: MouseButton) -> bool {
        self.pressed_mouse_buttons
            .iter()
            .any(|&mb| mb == mouse_button)
    }

    /// Returns an iterator over all pressed scan codes
    pub fn scan_codes_that_are_down(&self) -> impl Iterator<Item=u32> + '_ {
        self.pressed_keys
            .iter()
            .map((|k| k.1) as fn(&(VirtualKeyCode, u32)) -> u32)
    }

    /// Checks if the key corresponding to a scan code is down.
    pub fn scan_code_is_down(&self, scan_code: u32) -> bool {
        self.pressed_keys.iter().any(|&k| k.1 == scan_code)
    }

    /// Gets the current mouse position.
    ///
    /// this method can return None, either if no mouse is connected, or if no mouse events have
    /// been recorded
    pub fn mouse_position(&self) -> Option<(f64, f64)> {
        self.mouse_position
    }

    /// Returns an iterator over all buttons that are down.
    pub fn buttons_that_are_down<'a>(&self) -> impl Iterator<Item=Button> + '_ {
        let mouse_buttons = self.pressed_mouse_buttons
            .iter()
            .map((|&mb| Button::Mouse(mb)) as fn(&MouseButton) -> Button);
        let keys = self.pressed_keys.iter().flat_map(
            (|v| KeyThenCode::new(v.clone())) as fn(&(VirtualKeyCode, u32)) -> KeyThenCode,
        );
        mouse_buttons.chain(keys)
    }

    /// Checks if a button is down.
    pub fn button_is_down(&self, button: Button) -> bool {
        match button {
            Button::Key(k) => self.key_is_down(k),
            Button::Mouse(b) => self.mouse_button_is_down(b),
            Button::ScanCode(s) => self.scan_code_is_down(s),
        }
    }

    /// Returns the value of an axis by the string id, if the id doesn't exist this returns None.
    pub fn axis_value<T: Hash + Eq + ?Sized>(&self, id: &T) -> Option<f64>
    where
        AX: Borrow<T>,
    {
        self.bindings.axes.get(id).map(|a| {
            let pos = self.button_is_down(a.pos);
            let neg = self.button_is_down(a.neg);
            if pos == neg {
                0.0
            } else if pos {
                1.0
            } else {
                -1.0
            }
        })
    }

    /// Returns true if any of the action keys are down.
    pub fn action_is_down<T: Hash + Eq + ?Sized>(&self, action: &T) -> Option<bool>
    where
        AC: Borrow<T>,
    {
        self.bindings
            .actions
            .get(action)
            .map(|ref buttons| buttons.iter().any(|&b| self.button_is_down(b)))
    }
}
