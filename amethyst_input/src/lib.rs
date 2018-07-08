extern crate amethyst_config;
extern crate amethyst_core;
#[macro_use]
extern crate derivative;
extern crate fnv;
#[macro_use]
extern crate serde;
extern crate smallvec;
extern crate winit;

#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use self::axis::Axis;
pub use self::bindings::Bindings;
pub use self::bundle::InputBundle;
pub use self::button::Button;
pub use self::event::InputEvent;
pub use self::input_handler::InputHandler;
pub use self::system::InputSystem;
pub use self::util::{get_key, is_close_requested, is_key};

use std::iter::{Chain, FlatMap, Iterator, Map};
use std::slice::Iter;

use winit::{MouseButton, VirtualKeyCode};

mod axis;
mod bindings;
mod bundle;
mod button;
mod event;
mod input_handler;
mod local_mouse_button;
mod local_virtual_key_code;
mod system;
mod util;

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
