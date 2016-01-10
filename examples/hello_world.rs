//! The most basic Amethyst example.

extern crate amethyst;

use amethyst::{Actions, Application, Duration, State};

struct Example;

impl State for Example {
    fn new() -> Example {
        Example
    }

    fn on_start(&mut self) {
        println!("Begin!");
    }

    fn update(&mut self, _delta: Duration, game: &mut Actions) {
        println!("Hello from Amethyst!");
        game.quit();
    }

    fn on_stop(&mut self) {
        println!("End!");
    }
}

fn main() {
    let mut game = Application::new(Example);
    game.run();
}

