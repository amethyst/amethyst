//! The simplest Amethyst example.

extern crate amethyst;

use amethyst::prelude::*;
use amethyst::ecs::DispatcherBuilder;

struct Example;

impl State for Example {
    fn on_start<'a, 'b>(&mut self, _: &mut Engine, _: &mut Scene) -> Option<DispatcherBuilder<'a, 'b>> {
        println!("Begin!");
        None
    }

    fn update(&mut self, _: &mut Engine, _: &mut Scene) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut Engine, _: &mut Scene) {
        println!("End!");
    }
}

fn main() {
    let mut game = Application::new(Example).expect("Fatal error");
    game.run();
}
