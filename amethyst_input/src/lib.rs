extern crate serde;
extern crate winit;
extern crate fnv;
extern crate smallvec;

#[macro_use]
extern crate serde_derive;

pub use self::axis::Axis;
pub use self::bindings::Bindings;
pub use self::button::{Button, ButtonState, ChangeState};
pub use self::button::ButtonState::*;
pub use self::button::ChangeState::*;
pub use self::input_handler::InputHandler;

use std::iter::{Chain, Map, Iterator};
use std::slice::Iter;

use winit::{VirtualKeyCode, MouseButton};

mod axis;
mod bindings;
mod button;
mod input_handler;
mod local_mouse_button;
mod local_virtual_key_code;

/// Iterator over keycodes
pub type KeyCodes<'a> = Iter<'a, VirtualKeyCode>;

/// Iterator over MouseButtons
pub type MouseButtons<'a> = Iter<'a, MouseButton>;

/// An iterator over buttons
pub struct Buttons<'a> {
    iterator: Chain<
        Map<Iter<'a, MouseButton>, fn(&MouseButton) -> Button>,
        Map<Iter<'a, VirtualKeyCode>, fn(&VirtualKeyCode) -> Button>,
    >,
}

impl<'a> Iterator for Buttons<'a> {
    type Item = Button;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}
