//! The most basic Amethyst example.

extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::{Context, Config};

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut Context) {
        println!("Begin!");
    }

    fn update(&mut self, _: &mut Context) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut Context) {
        println!("End!");
    }
}

fn main() {
    let config = Config::default();
    let mut game = Application::new(Example, config);
    game.run();
}
