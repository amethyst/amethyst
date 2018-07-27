//! World resource that handles all user input.

use std::borrow::Borrow;
use std::hash::Hash;

use amethyst_core::shrev::EventChannel;
use smallvec::SmallVec;
use winit::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};

use super::controller::{ControllerButton, ControllerEvent};
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
    pressed_controller_buttons: SmallVec<[(u32, ControllerButton); 12]>,
    /// Holds current state of all connected controller axes
    controller_axes: SmallVec<[(u32, ControllerAxis, f64); 24]>,
    /// A list of raw and mapped ids for currently connected controllers.
    /// First number represents mapped ID visible to the user code,
    /// while second is the ID used by incoming events.
    connected_controllers: SmallVec<[(u32, u32); 8]>,
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

    /// Updates the input handler with a new controller event.
    ///
    /// Called internally from SdlEventsSystem when using sdl_controller feature.
    /// You should invoke it in your system if you provide
    /// your own controller input implementation.
    pub fn send_controller_event(
        &mut self,
        event: &ControllerEvent,
        event_handler: &mut EventChannel<InputEvent<AC>>,
    ) {
        use self::ControllerEvent::*;

        match *event {
            ControllerAxisMoved { which, axis, value } => {
                if let Some(controller_id) = self.controller_idx_to_id(which) {
                    self.controller_axes
                        .iter_mut()
                        .find(|(id, a, _)| *id == controller_id && *a == axis)
                        .map(|entry| entry.2 = value)
                        .unwrap_or_else(|| {
                            self.controller_axes.push((controller_id, axis, value));
                        });
                    event_handler.single_write(event.into());
                }
            }
            ControllerButtonPressed { which, button } => {
                if let Some(controller_id) = self.controller_idx_to_id(which) {
                    if self.pressed_controller_buttons
                        .iter()
                        .all(|&(id, b)| id != controller_id || b != button)
                    {
                        self.pressed_controller_buttons
                            .push((controller_id, button));
                        event_handler.iter_write(
                            [
                                event.into(),
                                ButtonPressed(Button::Controller(controller_id, button)),
                            ].iter()
                                .cloned(),
                        );
                        for (k, v) in self.bindings.actions.iter() {
                            for &b in v {
                                if Button::Controller(controller_id, button) == b {
                                    event_handler.single_write(ActionPressed(k.clone()));
                                }
                            }
                        }
                    }
                }
            }
            ControllerButtonReleased { which, button } => {
                if let Some(controller_id) = self.controller_idx_to_id(which) {
                    let index = self.pressed_controller_buttons
                        .iter()
                        .position(|&(id, b)| id == controller_id && b == button);
                    if let Some(i) = index {
                        self.pressed_controller_buttons.swap_remove(i);
                        event_handler.iter_write(
                            [
                                event.into(),
                                ButtonReleased(Button::Controller(controller_id, button)),
                            ].iter()
                                .cloned(),
                        );
                        for (k, v) in self.bindings.actions.iter() {
                            for &b in v {
                                if Button::Controller(controller_id, button) == b {
                                    event_handler.single_write(ActionReleased(k.clone()));
                                }
                            }
                        }
                    }
                }
            }
            ControllerConnected { which } => {
                if self.controller_idx_to_id(which).is_none() {
                    let controller_id = self.alloc_controller_id();
                    if self.connected_controllers
                        .iter()
                        .all(|&ids| ids.0 != controller_id)
                    {
                        self.connected_controllers.push((controller_id, which));
                    }
                }
            }
            ControllerDisconnected { which } => {
                if let Some(controller_id) = self.controller_idx_to_id(which) {
                    let index = self.connected_controllers
                        .iter()
                        .position(|&ids| ids.0 == controller_id);
                    if let Some(i) = index {
                        self.connected_controllers.swap_remove(i);
                        self.controller_axes.retain(|a| a.0 != controller_id);
                        self.pressed_controller_buttons
                            .retain(|b| b.0 != controller_id);
                    }
                }
            }
        }
    }

    /// Returns an iterator over all keys that are down.
    pub fn keys_that_are_down(&self) -> impl Iterator<Item = VirtualKeyCode> + '_ {
        self.pressed_keys.iter().map(|k| k.0)
    }

    /// Checks if a key is down.
    pub fn key_is_down(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.iter().any(|&k| k.0 == key)
    }

    /// Returns an iterator over all pressed mouse buttons
    pub fn mouse_buttons_that_are_down(&self) -> impl Iterator<Item = &MouseButton> {
        self.pressed_mouse_buttons.iter()
    }

    /// Checks if a mouse button is down.
    pub fn mouse_button_is_down(&self, mouse_button: MouseButton) -> bool {
        self.pressed_mouse_buttons
            .iter()
            .any(|&mb| mb == mouse_button)
    }

    /// Returns an iterator over all pressed scan codes
    pub fn scan_codes_that_are_down(&self) -> impl Iterator<Item = u32> + '_ {
        self.pressed_keys.iter().map(|k| k.1)
    }

    /// Checks if the key corresponding to a scan code is down.
    pub fn scan_code_is_down(&self, scan_code: u32) -> bool {
        self.pressed_keys.iter().any(|&k| k.1 == scan_code)
    }

    /// Returns an iterator over all pressed controller buttons on all controllers.
    pub fn controller_buttons_that_are_down(
        &self,
    ) -> impl Iterator<Item = &(u32, ControllerButton)> + '_ {
        self.pressed_controller_buttons.iter()
    }

    /// Checks if a controller button is down on specific controller.
    pub fn controller_button_is_down(
        &self,
        controller_id: u32,
        controller_button: ControllerButton,
    ) -> bool {
        self.pressed_controller_buttons
            .iter()
            .any(|&(id, b)| id == controller_id && b == controller_button)
    }

    /// List controller ids of all currently connected controllers.
    /// IDs are assigned sequentially in the order of connection
    /// starting from 0, always taking the lowest next free number.
    pub fn connected_controllers(&self) -> impl Iterator<Item = u32> + '_ {
        self.connected_controllers.iter().map(|ids| ids.0)
    }

    pub fn is_controller_connected(&self, controller_id: u32) -> bool {
        self.connected_controllers
            .iter()
            .any(|ids| ids.0 == controller_id)
    }

    /// Gets the current mouse position.
    ///
    /// this method can return None, either if no mouse is connected, or if no mouse events have
    /// been recorded
    pub fn mouse_position(&self) -> Option<(f64, f64)> {
        self.mouse_position
    }

    /// Returns an iterator over all buttons that are down.
    pub fn buttons_that_are_down<'a>(&self) -> impl Iterator<Item = Button> + '_ {
        let mouse_buttons = self.pressed_mouse_buttons
            .iter()
            .map(|&mb| Button::Mouse(mb));
        let keys = self.pressed_keys
            .iter()
            .flat_map(|v| KeyThenCode::new(v.clone()));
        let controller_buttons = self.pressed_controller_buttons
            .iter()
            .map(|&gb| Button::Controller(gb.0, gb.1));

        mouse_buttons.chain(keys).chain(controller_buttons)
    }

    /// Checks if a button is down.
    pub fn button_is_down(&self, button: Button) -> bool {
        match button {
            Button::Key(k) => self.key_is_down(k),
            Button::Mouse(b) => self.mouse_button_is_down(b),
            Button::ScanCode(s) => self.scan_code_is_down(s),
            Button::Controller(g, b) => self.controller_button_is_down(g, b),
        }
    }

    /// Returns the value of an axis by the string id, if the id doesn't exist this returns None.
    pub fn axis_value<T: Hash + Eq + ?Sized>(&self, id: &T) -> Option<f64>
    where
        AX: Borrow<T>,
    {
        self.bindings.axes.get(id).map(|a| match a {
            &Axis::Emulated { pos, neg, .. } => {
                let pos = self.button_is_down(pos);
                let neg = self.button_is_down(neg);
                if pos == neg {
                    0.0
                } else if pos {
                    1.0
                } else {
                    -1.0
                }
            }
            &Axis::Controller {
                controller_id,
                axis,
                invert,
                dead_zone,
                ..
            } => self.controller_axes
                .iter()
                .find(|&&(id, a, _)| id == controller_id && a == axis)
                .map(|&(_, _, val)| if invert { -val } else { val })
                .map(|val| {
                    if val < -dead_zone {
                        (val + dead_zone) / (1.0 - dead_zone)
                    } else if val > dead_zone {
                        (val - dead_zone) / (1.0 - dead_zone)
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.0),
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

    /// Retrieve next free controller number to allocate new controller to
    fn alloc_controller_id(&self) -> u32 {
        let mut i = 0u32;
        loop {
            if self.connected_controllers
                .iter()
                .find(|ids| ids.0 == i)
                .is_none()
            {
                return i;
            }
            i += 1;
        }
    }

    /// Map controller's index from external event into controller_id
    fn controller_idx_to_id(&self, index: u32) -> Option<u32> {
        self.connected_controllers
            .iter()
            .find(|ids| ids.1 == index)
            .map(|ids| ids.0)
    }
}
