//! The most basic Amethyst example.

extern crate amethyst;

use amethyst::engine::{Application, Duration, State, Trans};
use amethyst::context::{Context, Config};
use std::rc::Rc;
use std::cell::RefCell;

struct Example;

impl State for Example {
    fn on_start(&mut self) {
        println!("Begin!");
    }

    fn update(&mut self, _delta: Duration) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self) {
        println!("End!");
    }
}

fn main() {
    let config = Config::default();
    let context = Context::new(config).unwrap();
    let context_ref = Rc::new(RefCell::new(context));
    let mut game = Application::new(Example, context_ref);
    game.run();
}
