//! The simplest Amethyst example.

extern crate amethyst;

use amethyst::prelude::*;

struct Example;

impl State<()> for Example {
    fn on_start(&mut self, _: StateData<()>) {
        println!("Begin!");
    }

    fn on_stop(&mut self, _: StateData<()>) {
        println!("End!");
    }

    fn update(&mut self, _: StateData<()>) -> Trans<()> {
        println!("Hello from Amethyst!");
        Trans::Quit
    }
}

fn main() {
    amethyst::start_logger(Default::default());
    let mut game = Application::new("./", Example, ()).expect("Fatal error");
    game.run();
}
