//! The most basic Amethyst example.

extern crate amethyst;

use amethyst::{Application, Duration, State, Trans};

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
    let mut game = Application::new(Example);
    game.run();
}
