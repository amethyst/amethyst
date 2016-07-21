//! The most basic Amethyst example.

extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::context::{Context, Config};
use amethyst::ecs::{Planner, World};

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut Context, _: &mut World) {
        println!("Begin!");
    }

    fn update(&mut self, _: &mut Context, _: &mut World) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut Context, _: &mut World) {
        println!("End!");
    }
}

fn main() {
    let config = Config::default();
    let world = World::new();
    let planner = Planner::new(world, 1);
    let mut game = Application::new(Example, planner, config);
    game.run();
}
