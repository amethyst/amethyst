//! World resource that handles all user input.

use super::{
    controller::{ControllerButton, ControllerEvent},
    event::InputEvent::{self, *},
    scroll_direction::ScrollDirection,
    *,
};
use amethyst_core::shrev::EventChannel;
use derivative::Derivative;
use smallvec::SmallVec;
use std::{borrow::Borrow, hash::Hash};
use winit::{
    dpi::LogicalPosition, DeviceEvent, ElementState, Event, KeyboardInput, MouseButton,
    MouseScrollDelta, VirtualKeyCode, WindowEvent,
};

/// This struct holds state information about input devices.
///
/// For example, if a key is pressed on the keyboard, this struct will record
/// that the key is pressed until it is released again.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct InputHandler<T: BindingTypes> {
    /// Maps inputs to actions and axes.
    pub bindings: Bindings<T>,
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
    mouse_wheel_vertical: f64,
    mouse_wheel_horizontal: f64,
}

impl<T: BindingTypes> InputHandler<T> {
    /// Creates a new input handler.
    pub fn new() -> Self {
        Default::default()
    }

    /// Updates the input handler with a new engine event.
    ///
    /// The Amethyst game engine will automatically call this if the InputHandler is attached to
    /// the world as a resource.
    pub fn send_event(
        &mut self,
        event: &Event,
        event_handler: &mut EventChannel<InputEvent<T::Action>>,
        hidpi: f64,
    ) {
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
                } => {
                    if self.pressed_keys.iter().all(|&k| k.0 != key_code) {
                        self.pressed_keys.push((key_code, scancode));
                        event_handler.iter_write(
                            [
                                KeyPressed { key_code, scancode },
                                ButtonPressed(Button::Key(key_code)),
                                ButtonPressed(Button::ScanCode(scancode)),
                            ]
                            .iter()
                            .cloned(),
                        );
                        for (action, combinations) in self.bindings.actions.iter() {
                            for combination in combinations.iter().filter(|c| {
                                c.contains(&Button::Key(key_code))
                                    || c.contains(&Button::ScanCode(scancode))
                            }) {
                                if combination
                                    .iter()
                                    .all(|button| self.button_is_down(*button))
                                {
                                    event_handler.single_write(ActionPressed(action.clone()));
                                }
                            }
                        }
                    }
                }
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
                            ]
                            .iter()
                            .cloned(),
                        );
                        for (action, combinations) in self.bindings.actions.iter() {
                            for combination in combinations {
                                if combination.contains(&Button::Key(key_code))
                                    && combination
                                        .iter()
                                        .filter(|b| b != &&Button::Key(key_code))
                                        .all(|b| self.button_is_down(*b))
                                {
                                    event_handler.single_write(ActionReleased(action.clone()));
                                }
                                if combination.contains(&Button::ScanCode(scancode))
                                    && combination
                                        .iter()
                                        .filter(|b| b != &&Button::ScanCode(scancode))
                                        .all(|b| self.button_is_down(*b))
                                {
                                    event_handler.single_write(ActionReleased(action.clone()));
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
                    if self
                        .pressed_mouse_buttons
                        .iter()
                        .all(|&b| b != mouse_button)
                    {
                        self.pressed_mouse_buttons.push(mouse_button);
                        event_handler.iter_write(
                            [
                                MouseButtonPressed(mouse_button),
                                ButtonPressed(Button::Mouse(mouse_button)),
                            ]
                            .iter()
                            .cloned(),
                        );
                        for (action, combinations) in self.bindings.actions.iter() {
                            for combination in combinations
                                .iter()
                                .filter(|c| c.contains(&Button::Mouse(mouse_button)))
                            {
                                if combination
                                    .iter()
                                    .all(|button| self.button_is_down(*button))
                                {
                                    event_handler.single_write(ActionPressed(action.clone()));
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
                    let index = self
                        .pressed_mouse_buttons
                        .iter()
                        .position(|&b| b == mouse_button);
                    if let Some(i) = index {
                        self.pressed_mouse_buttons.swap_remove(i);
                        event_handler.iter_write(
                            [
                                MouseButtonReleased(mouse_button),
                                ButtonReleased(Button::Mouse(mouse_button)),
                            ]
                            .iter()
                            .cloned(),
                        );
                        for (action, combinations) in self.bindings.actions.iter() {
                            for combination in combinations {
                                if combination.contains(&Button::Mouse(mouse_button))
                                    && combination
                                        .iter()
                                        .filter(|b| b != &&Button::Mouse(mouse_button))
                                        .all(|b| self.button_is_down(*b))
                                {
                                    event_handler.single_write(ActionReleased(action.clone()));
                                }
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved {
                    position: LogicalPosition { x, y },
                    ..
                } => {
                    if let Some((old_x, old_y)) = self.mouse_position {
                        event_handler.single_write(CursorMoved {
                            delta_x: x * hidpi - old_x,
                            delta_y: y * hidpi - old_y,
                        });
                    }
                    self.mouse_position = Some((x * hidpi, y * hidpi));
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
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(delta_x, delta_y),
                } => {
                    if delta_x != 0.0 {
                        self.mouse_wheel_horizontal = delta_x.signum().into();
                    }
                    if delta_y != 0.0 {
                        self.mouse_wheel_vertical = delta_y.signum().into();
                    }
                    self.invoke_wheel_moved(delta_x.into(), delta_y.into(), event_handler);
                }
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::PixelDelta(LogicalPosition { x, y }),
                } => {
                    if x != 0.0 {
                        self.mouse_wheel_horizontal = x.signum();
                    }
                    if y != 0.0 {
                        self.mouse_wheel_vertical = y.signum();
                    }
                    self.invoke_wheel_moved(x, y, event_handler);
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
        event_handler: &mut EventChannel<InputEvent<T::Action>>,
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
                    if self
                        .pressed_controller_buttons
                        .iter()
                        .all(|&(id, b)| id != controller_id || b != button)
                    {
                        self.pressed_controller_buttons
                            .push((controller_id, button));
                        event_handler.iter_write(
                            [
                                event.into(),
                                ButtonPressed(Button::Controller(controller_id, button)),
                            ]
                            .iter()
                            .cloned(),
                        );
                        for (action, combinations) in self.bindings.actions.iter() {
                            for combination in combinations
                                .iter()
                                .filter(|c| c.contains(&Button::Controller(controller_id, button)))
                            {
                                if combination
                                    .iter()
                                    .all(|button| self.button_is_down(*button))
                                {
                                    event_handler.single_write(ActionPressed(action.clone()));
                                }
                            }
                        }
                    }
                }
            }
            ControllerButtonReleased { which, button } => {
                if let Some(controller_id) = self.controller_idx_to_id(which) {
                    let index = self
                        .pressed_controller_buttons
                        .iter()
                        .position(|&(id, b)| id == controller_id && b == button);
                    if let Some(i) = index {
                        self.pressed_controller_buttons.swap_remove(i);
                        event_handler.iter_write(
                            [
                                event.into(),
                                ButtonReleased(Button::Controller(controller_id, button)),
                            ]
                            .iter()
                            .cloned(),
                        );
                        for (action, combinations) in self.bindings.actions.iter() {
                            for combination in combinations {
                                if combination.contains(&Button::Controller(controller_id, button))
                                    && combination
                                        .iter()
                                        .filter(|b| {
                                            b != &&Button::Controller(controller_id, button)
                                        })
                                        .all(|b| self.button_is_down(*b))
                                {
                                    event_handler.single_write(ActionReleased(action.clone()));
                                }
                            }
                        }
                    }
                }
            }
            ControllerConnected { which } => {
                if self.controller_idx_to_id(which).is_none() {
                    let controller_id = self.alloc_controller_id();
                    if self
                        .connected_controllers
                        .iter()
                        .all(|&ids| ids.0 != controller_id)
                    {
                        self.connected_controllers.push((controller_id, which));
                    }
                }
            }
            ControllerDisconnected { which } => {
                if let Some(controller_id) = self.controller_idx_to_id(which) {
                    let index = self
                        .connected_controllers
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

    /// This function is to be called whenever a frame begins. It resets some input values.
    ///
    /// The `InputSystem` will call this automatically. If you're using that system, you
    /// don't need to call this function.
    pub fn send_frame_begin(&mut self) {
        self.mouse_wheel_vertical = 0.0;
        self.mouse_wheel_horizontal = 0.0;
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

    /// If the mouse wheel was scrolled this frame this function will return the direction it was scrolled.
    ///
    /// If "horizontal" is true this will return the horizontal mouse value. You almost always want the
    /// vertical mouse value.
    pub fn mouse_wheel_value(&self, horizontal: bool) -> f64 {
        if horizontal {
            self.mouse_wheel_horizontal
        } else {
            self.mouse_wheel_vertical
        }
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

    /// Returns true if a controller with the given id is connected.
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
        let mouse_buttons = self
            .pressed_mouse_buttons
            .iter()
            .map(|&mb| Button::Mouse(mb));
        let keys = self.pressed_keys.iter().flat_map(|v| KeyThenCode::new(*v));
        let controller_buttons = self
            .pressed_controller_buttons
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
            _ => false,
        }
    }

    /// Returns the value of an axis by the string id, if the id doesn't exist this returns None.
    pub fn axis_value<A>(&self, id: &A) -> Option<f64>
    where
        T::Axis: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.bindings.axes.get(id).map(|a| match *a {
            Axis::Emulated { pos, neg, .. } => {
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
            Axis::Controller {
                controller_id,
                axis,
                invert,
                dead_zone,
                ..
            } => self
                .controller_axes
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
            Axis::MouseWheel { horizontal } => self.mouse_wheel_value(horizontal),
        })
    }

    /// Returns true if any of the actions bindings is down.
    ///
    /// If a binding represents a combination of buttons, all of them need to be down.
    pub fn action_is_down<A>(&self, action: &A) -> Option<bool>
    where
        T::Action: Borrow<A>,
        A: Hash + Eq + ?Sized,
    {
        self.bindings.actions.get(action).map(|combinations| {
            combinations.iter().any(|combination| {
                combination
                    .iter()
                    .all(|button| self.button_is_down(*button))
            })
        })
    }

    /// Retrieve next free controller number to allocate new controller to
    fn alloc_controller_id(&self) -> u32 {
        let mut i = 0u32;
        loop {
            if self
                .connected_controllers
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

    /// Iterates all input bindings and invokes ActionWheelMoved for each action bound to the mouse wheel
    fn invoke_wheel_moved(
        &self,
        delta_x: f64,
        delta_y: f64,
        event_handler: &mut EventChannel<InputEvent<T::Action>>,
    ) {
        let mut events = Vec::<InputEvent<T::Action>>::new();

        // determine if a horizontal scroll happend
        let dir_x = match delta_x {
            dx if dx > 0.0 => {
                events.push(MouseWheelMoved(ScrollDirection::ScrollRight));
                Some(ScrollDirection::ScrollRight)
            }
            dx if dx < 0.0 => {
                events.push(MouseWheelMoved(ScrollDirection::ScrollLeft));
                Some(ScrollDirection::ScrollLeft)
            }
            _ => None,
        };

        // determine if a vertical scroll happend
        let dir_y = match delta_y {
            dy if dy > 0.0 => {
                events.push(MouseWheelMoved(ScrollDirection::ScrollDown));
                Some(ScrollDirection::ScrollDown)
            }
            dy if dy < 0.0 => {
                events.push(MouseWheelMoved(ScrollDirection::ScrollUp));
                Some(ScrollDirection::ScrollUp)
            }
            _ => None,
        };

        // check for actions being bound to any invoked mouse wheel
        for (action, combinations) in self.bindings.actions.iter() {
            for ref combination in combinations {
                if let Some(dir) = dir_x {
                    if combination.contains(&Button::MouseWheel(dir))
                        && combination
                            .iter()
                            .filter(|b| **b != Button::MouseWheel(dir))
                            .all(|b| self.button_is_down(*b))
                    {
                        events.push(ActionWheelMoved(action.clone()));
                    }
                }
                if let Some(dir) = dir_y {
                    if combination.contains(&Button::MouseWheel(dir))
                        && combination
                            .iter()
                            .filter(|b| **b != Button::MouseWheel(dir))
                            .all(|b| self.button_is_down(*b))
                    {
                        events.push(ActionWheelMoved(action.clone()));
                    }
                }
            }
        }

        // send all collected events
        event_handler.iter_write(events);
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;
    use winit::{
        DeviceId, ElementState, Event, KeyboardInput, ModifiersState, ScanCode, WindowEvent,
        WindowId,
    };

    const HIDPI: f64 = 1.0;

    #[test]
    fn key_action_response() {
        // Register an action triggered by a key
        // Press the key and check for a press event of both the key and the action.
        // Release the key and check for a release event of both the key and the action.

        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        let mut reader = events.register_reader();
        handler
            .bindings
            .insert_action_binding(
                String::from("test_key_action"),
                [Button::Key(VirtualKeyCode::Up)].iter().cloned(),
            )
            .unwrap();
        assert_eq!(handler.action_is_down("test_key_action"), Some(false));
        handler.send_event(&key_press(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_key_action"), Some(true));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                InputEvent::ActionPressed(String::from("test_key_action")),
                InputEvent::KeyPressed {
                    key_code: VirtualKeyCode::Up,
                    scancode: 104,
                },
                InputEvent::ButtonPressed(Button::Key(VirtualKeyCode::Up)),
                InputEvent::ButtonPressed(Button::ScanCode(104)),
            ],
        );
        handler.send_event(&key_release(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_key_action"), Some(false));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                InputEvent::ActionReleased(String::from("test_key_action")),
                InputEvent::KeyReleased {
                    key_code: VirtualKeyCode::Up,
                    scancode: 104,
                },
                InputEvent::ButtonReleased(Button::Key(VirtualKeyCode::Up)),
                InputEvent::ButtonReleased(Button::ScanCode(104)),
            ],
        );
    }

    #[test]
    fn mouse_action_response() {
        // Register an action triggered by a mouse button
        // Press the button and check for a press event of both the button and the action.
        // Release the button and check for a release event of both the button and the action.

        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        let mut reader = events.register_reader();
        handler
            .bindings
            .insert_action_binding(
                String::from("test_mouse_action"),
                [Button::Mouse(MouseButton::Left)].iter().cloned(),
            )
            .unwrap();
        assert_eq!(handler.action_is_down("test_mouse_action"), Some(false));
        handler.send_event(&mouse_press(MouseButton::Left), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_mouse_action"), Some(true));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                InputEvent::ActionPressed(String::from("test_mouse_action")),
                InputEvent::MouseButtonPressed(MouseButton::Left),
                InputEvent::ButtonPressed(Button::Mouse(MouseButton::Left)),
            ],
        );
        handler.send_event(&mouse_release(MouseButton::Left), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_mouse_action"), Some(false));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                InputEvent::ActionReleased(String::from("test_mouse_action")),
                InputEvent::MouseButtonReleased(MouseButton::Left),
                InputEvent::ButtonReleased(Button::Mouse(MouseButton::Left)),
            ],
        );
    }

    #[test]
    fn combo_action_response() {
        // Register a combo
        // Press one key in the combo, make sure we get the key press but no action event
        // Press the second key in the combo, we should get both key press and action event
        // Release first key, we should get key release and action release
        // Release second key, we should key release and no action release

        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        let mut reader = events.register_reader();
        handler
            .bindings
            .insert_action_binding(
                String::from("test_combo_action"),
                [
                    Button::Key(VirtualKeyCode::Up),
                    Button::Key(VirtualKeyCode::Down),
                ]
                .iter()
                .cloned(),
            )
            .unwrap();
        assert_eq!(handler.action_is_down("test_combo_action"), Some(false));
        handler.send_event(&key_press(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_combo_action"), Some(false));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                InputEvent::KeyPressed {
                    key_code: VirtualKeyCode::Up,
                    scancode: 104,
                },
                InputEvent::ButtonPressed(Button::Key(VirtualKeyCode::Up)),
                InputEvent::ButtonPressed(Button::ScanCode(104)),
            ],
        );
        handler.send_event(&key_press(112, VirtualKeyCode::Down), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_combo_action"), Some(true));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                ActionPressed(String::from("test_combo_action")),
                InputEvent::KeyPressed {
                    key_code: VirtualKeyCode::Down,
                    scancode: 112,
                },
                InputEvent::ButtonPressed(Button::Key(VirtualKeyCode::Down)),
                InputEvent::ButtonPressed(Button::ScanCode(112)),
            ],
        );
        handler.send_event(&key_release(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_combo_action"), Some(false));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                InputEvent::ActionReleased(String::from("test_combo_action")),
                InputEvent::KeyReleased {
                    key_code: VirtualKeyCode::Up,
                    scancode: 104,
                },
                InputEvent::ButtonReleased(Button::Key(VirtualKeyCode::Up)),
                InputEvent::ButtonReleased(Button::ScanCode(104)),
            ],
        );

        handler.send_event(&key_release(112, VirtualKeyCode::Down), &mut events, HIDPI);
        assert_eq!(handler.action_is_down("test_combo_action"), Some(false));
        let event_vec = events.read(&mut reader).cloned().collect::<Vec<_>>();
        sets_are_equal(
            &event_vec,
            &[
                InputEvent::KeyReleased {
                    key_code: VirtualKeyCode::Down,
                    scancode: 112,
                },
                InputEvent::ButtonReleased(Button::Key(VirtualKeyCode::Down)),
                InputEvent::ButtonReleased(Button::ScanCode(112)),
            ],
        );
    }

    #[test]
    fn emulated_axis_response() {
        // Register an axis triggered by two keys
        // Check that with nothing pressed we return 0.
        // Press the positive and check for a positive response
        // Release the positive, press the negative and check for a negative respones
        // Press both and check for 0.
        // Release both and check for 0.

        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        handler
            .bindings
            .insert_axis(
                String::from("test_axis"),
                Axis::Emulated {
                    pos: Button::Key(VirtualKeyCode::Up),
                    neg: Button::Key(VirtualKeyCode::Down),
                },
            )
            .unwrap();
        assert_eq!(handler.axis_value("test_axis"), Some(0.0));
        handler.send_event(&key_press(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert_eq!(handler.axis_value("test_axis"), Some(1.0));
        handler.send_event(&key_release(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert_eq!(handler.axis_value("test_axis"), Some(0.0));
        handler.send_event(&key_press(112, VirtualKeyCode::Down), &mut events, HIDPI);
        assert_eq!(handler.axis_value("test_axis"), Some(-1.0));
        handler.send_event(&key_press(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert_eq!(handler.axis_value("test_axis"), Some(0.0));
        handler.send_event(&key_release(104, VirtualKeyCode::Up), &mut events, HIDPI);
        handler.send_event(&key_release(112, VirtualKeyCode::Down), &mut events, HIDPI);
        assert_eq!(handler.axis_value("test_axis"), Some(0.0));
    }

    #[test]
    fn pressed_iter_response() {
        // Press some buttons and make sure the input handler returns them
        // in iterators

        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        assert_eq!(handler.keys_that_are_down().next(), None);
        assert_eq!(handler.scan_codes_that_are_down().next(), None);
        assert_eq!(handler.mouse_buttons_that_are_down().next(), None);
        assert_eq!(handler.buttons_that_are_down().next(), None);
        handler.send_event(&key_press(104, VirtualKeyCode::Up), &mut events, HIDPI);
        handler.send_event(&key_press(112, VirtualKeyCode::Down), &mut events, HIDPI);
        handler.send_event(&key_press(75, VirtualKeyCode::Left), &mut events, HIDPI);
        handler.send_event(&key_press(109, VirtualKeyCode::Right), &mut events, HIDPI);
        handler.send_event(&mouse_press(MouseButton::Left), &mut events, HIDPI);
        handler.send_event(&mouse_press(MouseButton::Right), &mut events, HIDPI);
        sets_are_equal(
            &handler.keys_that_are_down().collect::<Vec<_>>(),
            &[
                VirtualKeyCode::Up,
                VirtualKeyCode::Down,
                VirtualKeyCode::Left,
                VirtualKeyCode::Right,
            ],
        );
        sets_are_equal(
            &handler.scan_codes_that_are_down().collect::<Vec<_>>(),
            &[104, 112, 75, 109],
        );
        sets_are_equal(
            &handler.mouse_buttons_that_are_down().collect::<Vec<_>>(),
            &[&MouseButton::Left, &MouseButton::Right],
        );
        sets_are_equal(
            &handler.buttons_that_are_down().collect::<Vec<_>>(),
            &[
                Button::Key(VirtualKeyCode::Up),
                Button::Key(VirtualKeyCode::Down),
                Button::Key(VirtualKeyCode::Left),
                Button::Key(VirtualKeyCode::Right),
                Button::ScanCode(104),
                Button::ScanCode(112),
                Button::ScanCode(75),
                Button::ScanCode(109),
                Button::Mouse(MouseButton::Left),
                Button::Mouse(MouseButton::Right),
            ],
        );
        handler.send_event(&key_release(104, VirtualKeyCode::Up), &mut events, HIDPI);
        sets_are_equal(
            &handler.keys_that_are_down().collect::<Vec<_>>(),
            &[
                VirtualKeyCode::Down,
                VirtualKeyCode::Left,
                VirtualKeyCode::Right,
            ],
        );
        sets_are_equal(
            &handler.scan_codes_that_are_down().collect::<Vec<_>>(),
            &[112, 75, 109],
        );
        handler.send_event(&key_release(109, VirtualKeyCode::Right), &mut events, HIDPI);
        sets_are_equal(
            &handler.keys_that_are_down().collect::<Vec<_>>(),
            &[VirtualKeyCode::Down, VirtualKeyCode::Left],
        );
        sets_are_equal(
            &handler.scan_codes_that_are_down().collect::<Vec<_>>(),
            &[112, 75],
        );
        handler.send_event(&key_release(112, VirtualKeyCode::Down), &mut events, HIDPI);
        sets_are_equal(
            &handler.keys_that_are_down().collect::<Vec<_>>(),
            &[VirtualKeyCode::Left],
        );
        sets_are_equal(
            &handler.scan_codes_that_are_down().collect::<Vec<_>>(),
            &[75],
        );
        handler.send_event(&key_release(75, VirtualKeyCode::Left), &mut events, HIDPI);
        assert_eq!(handler.keys_that_are_down().next(), None);
        assert_eq!(handler.scan_codes_that_are_down().next(), None);
        sets_are_equal(
            &handler.buttons_that_are_down().collect::<Vec<_>>(),
            &[
                Button::Mouse(MouseButton::Left),
                Button::Mouse(MouseButton::Right),
            ],
        );
        handler.send_event(&mouse_release(MouseButton::Left), &mut events, HIDPI);
        sets_are_equal(
            &handler.buttons_that_are_down().collect::<Vec<_>>(),
            &[Button::Mouse(MouseButton::Right)],
        );
        handler.send_event(&mouse_release(MouseButton::Right), &mut events, HIDPI);
        assert_eq!(handler.buttons_that_are_down().next(), None);
    }

    #[test]
    fn basic_key_check() {
        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        assert!(!handler.key_is_down(VirtualKeyCode::Up));
        assert!(!handler.scan_code_is_down(104));
        assert!(!handler.button_is_down(Button::Key(VirtualKeyCode::Up)));
        assert!(!handler.button_is_down(Button::ScanCode(104)));
        handler.send_event(&key_press(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert!(handler.key_is_down(VirtualKeyCode::Up));
        assert!(handler.scan_code_is_down(104));
        assert!(handler.button_is_down(Button::Key(VirtualKeyCode::Up)));
        assert!(handler.button_is_down(Button::ScanCode(104)));
        handler.send_event(&key_release(104, VirtualKeyCode::Up), &mut events, HIDPI);
        assert!(!handler.key_is_down(VirtualKeyCode::Up));
        assert!(!handler.scan_code_is_down(104));
        assert!(!handler.button_is_down(Button::Key(VirtualKeyCode::Up)));
        assert!(!handler.button_is_down(Button::ScanCode(104)));
    }

    #[test]
    fn basic_mouse_check() {
        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        assert!(!handler.mouse_button_is_down(MouseButton::Left));
        assert!(!handler.button_is_down(Button::Mouse(MouseButton::Left)));
        handler.send_event(&mouse_press(MouseButton::Left), &mut events, HIDPI);
        assert!(handler.mouse_button_is_down(MouseButton::Left));
        assert!(handler.button_is_down(Button::Mouse(MouseButton::Left)));
        handler.send_event(&mouse_release(MouseButton::Left), &mut events, HIDPI);
        assert!(!handler.mouse_button_is_down(MouseButton::Left));
        assert!(!handler.button_is_down(Button::Mouse(MouseButton::Left)));
    }

    #[test]
    fn basic_mouse_wheel_check() {
        let mut handler = InputHandler::<StringBindings>::new();
        let mut events = EventChannel::<InputEvent<String>>::new();
        assert_eq!(handler.mouse_wheel_value(false), 0.0);
        assert_eq!(handler.mouse_wheel_value(true), 0.0);
        handler.send_event(&mouse_wheel(0.0, 5.0), &mut events, HIDPI);
        assert_eq!(handler.mouse_wheel_value(false), 1.0);
        assert_eq!(handler.mouse_wheel_value(true), 0.0);
        handler.send_frame_begin();
        assert_eq!(handler.mouse_wheel_value(false), 0.0);
        assert_eq!(handler.mouse_wheel_value(true), 0.0);
        handler.send_event(&mouse_wheel(5.0, 0.0), &mut events, HIDPI);
        assert_eq!(handler.mouse_wheel_value(false), 0.0);
        assert_eq!(handler.mouse_wheel_value(true), 1.0);
        handler.send_frame_begin();
        assert_eq!(handler.mouse_wheel_value(false), 0.0);
        assert_eq!(handler.mouse_wheel_value(true), 0.0);
        handler.send_event(&mouse_wheel(0.0, -5.0), &mut events, HIDPI);
        assert_eq!(handler.mouse_wheel_value(false), -1.0);
        assert_eq!(handler.mouse_wheel_value(true), 0.0);
        handler.send_frame_begin();
        assert_eq!(handler.mouse_wheel_value(false), 0.0);
        assert_eq!(handler.mouse_wheel_value(true), 0.0);
        handler.send_event(&mouse_wheel(-5.0, 0.0), &mut events, HIDPI);
        assert_eq!(handler.mouse_wheel_value(false), 0.0);
        assert_eq!(handler.mouse_wheel_value(true), -1.0);
    }

    /// Compares two sets for equality, but not the order
    fn sets_are_equal<T>(a: &[T], b: &[T])
    where
        T: PartialEq<T> + Debug,
    {
        let mut ret = a.len() == b.len();

        if ret {
            let mut b = b.iter().collect::<Vec<_>>();
            for a in a.iter() {
                if let Some(i) = b.iter().position(|b| a == *b) {
                    b.swap_remove(i);
                } else {
                    ret = false;
                    break;
                }
            }
        };
        if !ret {
            panic!(
                "assertion failed: `(left == right)`
left: `{:?}`
right: `{:?}`",
                a, b
            );
        }
    }

    fn key_press(scancode: ScanCode, virtual_keycode: VirtualKeyCode) -> Event {
        key_event(scancode, virtual_keycode, ElementState::Pressed)
    }

    fn key_release(scancode: ScanCode, virtual_keycode: VirtualKeyCode) -> Event {
        key_event(scancode, virtual_keycode, ElementState::Released)
    }

    fn key_event(
        scancode: ScanCode,
        virtual_keycode: VirtualKeyCode,
        state: ElementState,
    ) -> Event {
        Event::WindowEvent {
            window_id: unsafe { WindowId::dummy() },
            event: WindowEvent::KeyboardInput {
                device_id: unsafe { DeviceId::dummy() },
                input: KeyboardInput {
                    scancode,
                    state,
                    virtual_keycode: Some(virtual_keycode),
                    modifiers: ModifiersState {
                        shift: false,
                        ctrl: false,
                        alt: false,
                        logo: false,
                    },
                },
            },
        }
    }

    fn mouse_press(button: MouseButton) -> Event {
        mouse_event(button, ElementState::Pressed)
    }

    fn mouse_release(button: MouseButton) -> Event {
        mouse_event(button, ElementState::Released)
    }

    fn mouse_event(button: MouseButton, state: ElementState) -> Event {
        Event::WindowEvent {
            window_id: unsafe { WindowId::dummy() },
            event: WindowEvent::MouseInput {
                device_id: unsafe { DeviceId::dummy() },
                state,
                button,
                modifiers: ModifiersState {
                    shift: false,
                    ctrl: false,
                    alt: false,
                    logo: false,
                },
            },
        }
    }

    fn mouse_wheel(x: f32, y: f32) -> Event {
        Event::DeviceEvent {
            device_id: unsafe { DeviceId::dummy() },
            event: DeviceEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(x, y),
            },
        }
    }
}
