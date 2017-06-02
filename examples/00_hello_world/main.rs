//! The simplest Amethyst example.

extern crate amethyst;

use amethyst::{Application, Engine, State, Trans};
use amethyst::gfx_device::DisplayConfig;

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut Engine) {
        println!("Begin!");
    }

    fn update(&mut self, _: &mut Engine) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut Engine) {
        println!("End!");
    }
}

fn main() {
    let cfg = DisplayConfig::default();
    let mut game = Application::build(Example, cfg).done();
    game.run();
}
