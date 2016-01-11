//! The most basic Amethyst example.

extern crate amethyst;

use amethyst::{Application, Duration, State};

struct GameState;

impl State for GameState {
    fn new() -> GameState {
        GameState
    }

    fn on_start(&mut self) {
        println!("Begin!");
    }

    fn update(&mut self, _delta: Duration) {
        println!("Hello from Amethyst!");
        std::process::exit(0);
    }

    fn on_stop(&mut self) {
        println!("End!");
    }
}

fn main() {
    let mut game = Application::new(GameState::new());
    game.run();
}

