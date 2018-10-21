//! A collection of abstractions for various input devices to be used with Amethyst.
#![warn(missing_docs)]

extern crate amethyst_config;
extern crate amethyst_core;
#[macro_use]
extern crate derivative;
extern crate fnv;
#[macro_use]
extern crate serde;
extern crate smallvec;
extern crate winit;

#[cfg(feature = "sdl_controller")]
extern crate sdl2;

#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use self::axis::Axis;
pub use self::bindings::Bindings;
pub use self::bundle::InputBundle;
pub use self::button::Button;
pub use self::controller::{ControllerAxis, ControllerButton};
pub use self::event::InputEvent;
pub use self::input_handler::InputHandler;
pub use self::scroll_direction::ScrollDirection;
#[cfg(feature = "sdl_controller")]
pub use self::sdl_events_system::SdlEventsSystem;
pub use self::system::InputSystem;
pub use self::util::{get_input_axis_simple, get_key, is_close_requested, is_key_down};

use std::iter::Iterator;
use winit::VirtualKeyCode;

mod axis;
mod bindings;
mod bundle;
mod button;
mod controller;
mod event;
mod input_handler;
mod local_mouse_button;
mod local_virtual_key_code;
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
