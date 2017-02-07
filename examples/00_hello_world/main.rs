//! The simplest Amethyst example.

extern crate amethyst;

use amethyst::engine::{Application, State, Trans};
use amethyst::specs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::asset_manager::AssetManager;
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
    let display_config = DisplayConfig::default();
    let mut game = Application::build(Example, display_config).done();
    game.run();
}
