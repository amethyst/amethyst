//! A collection of abstractions for various input devices to be used with Amethyst.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[cfg(feature = "sdl_controller")]
pub use self::sdl_events_system::SdlEventsSystem;
pub use self::{
    axis::Axis,
    bindings::{BindingError, Bindings},
    bundle::{BindingsFileError, InputBundle},
    button::Button,
    controller::{ControllerAxis, ControllerButton, ControllerEvent},
    event::InputEvent,
    input_handler::InputHandler,
    scroll_direction::ScrollDirection,
    system::InputSystem,
    util::{get_input_axis_simple, get_key, is_close_requested, is_key_down},
};

use std::iter::Iterator;

use winit::{self, VirtualKeyCode};

mod axis;
mod bindings;
mod bundle;
mod button;
mod controller;
mod event;
mod input_handler;
mod scroll_direction;
mod system;
mod util;

#[cfg(feature = "sdl_controller")]
mod sdl_events_system;

struct KeyThenCode {
    value: (VirtualKeyCode, u32),
    index: u8,
}

impl KeyThenCode {
    pub fn new(value: (VirtualKeyCode, u32)) -> KeyThenCode {
        KeyThenCode { value, index: 0 }
    }
}

impl Iterator for KeyThenCode {
    type Item = Button;
    fn next(&mut self) -> Option<Button> {
        let index = self.index;
        if self.index < 2 {
            self.index += 1;
        }
        match index {
            0 => Some(Button::Key(self.value.0)),
            1 => Some(Button::ScanCode(self.value.1)),
            _ => None,
        }
    }
}
