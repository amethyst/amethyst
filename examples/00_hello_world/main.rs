//! The simplest Amethyst example.

extern crate amethyst;

use amethyst::{Application, State, Trans};
use amethyst::asset_manager::AssetManager;
use amethyst::ecs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::Pipeline;

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) {
        println!("Begin!");
    }

    fn update(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        println!("Hello from Amethyst!");
        Trans::Quit
    }

    fn on_stop(&mut self, _: &mut World, _: &mut AssetManager, _: &mut Pipeline) {
        println!("End!");
    }
}

fn main() {
    let cfg = DisplayConfig::default();
    let mut game = Application::build(Example, cfg).done();
    game.run();
}
