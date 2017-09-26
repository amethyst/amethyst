extern crate winit;
extern crate fnv;
extern crate smallvec;

#[macro_use]
extern crate serde_derive;
extern crate shrev;

pub use self::axis::Axis;
pub use self::bindings::Bindings;
pub use self::button::Button;
pub use self::event::InputEvent;
pub use self::input_handler::InputHandler;

use std::iter::{Chain, FlatMap, Map, Iterator};
use std::slice::Iter;

use winit::{VirtualKeyCode, MouseButton};

mod axis;
mod bindings;
mod button;
mod input_handler;
mod local_mouse_button;
mod local_virtual_key_code;
mod event;

// This entire set ot types is to be eliminated once impl Trait is released.

/// Iterator over keycodes
pub type KeyCodes<'a> = Map<Iter<'a, (VirtualKeyCode, u32)>, fn(&(VirtualKeyCode, u32)) -> VirtualKeyCode>;

/// Iterator over key scan codes
pub type ScanCodes<'a> = Map<Iter<'a, (VirtualKeyCode, u32)>, fn(&(VirtualKeyCode, u32)) -> u32>;

/// Iterator over MouseButtons
pub type MouseButtons<'a> = Iter<'a, MouseButton>;

/// An iterator over buttons
pub struct Buttons<'a> {
    iterator: Chain<
        Map<Iter<'a, MouseButton>, fn(&MouseButton) -> Button>,
        FlatMap<Iter<'a, (VirtualKeyCode, u32)>, KeyThenCode, fn(&(VirtualKeyCode, u32)) -> KeyThenCode>,
    >,
}

struct KeyThenCode {
    value: (VirtualKeyCode, u32),
    index: u8,
}

impl KeyThenCode {
    pub fn new(value: (VirtualKeyCode, u32)) -> KeyThenCode {
        KeyThenCode {
            value,
            index: 0,
        }
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

impl<'a> Iterator for Buttons<'a> {
    type Item = Button;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}
