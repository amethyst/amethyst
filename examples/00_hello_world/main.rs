//! The simplest Amethyst example.

extern crate amethyst;

use amethyst::{Application, State, Trans};
use amethyst::asset_manager::AssetManager;
use amethyst::ecs::World;

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut World, _: &mut AssetManager) {
        println!("Begin!");
    }

    fn update(&mut self, _: &mut World, _: &mut AssetManager) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut World, _: &mut AssetManager) {
        println!("End!");
    }
}

fn main() {
    let mut game = Application::build(Example).done();
    game.run();
}
