//! Opens an empty window.

extern crate amethyst;

use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::AssetManager;
use amethyst::config::Config;
use amethyst::ecs::World;

struct Example;

impl State for Example {
    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     _: &mut World,
                     _: &mut AssetManager)
                     -> Trans {
        for e in events {
            match **e {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        Trans::None
    }
}

fn main() {
    let path = format!("{}/examples/01_window/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let mut game = Application::build(Example).done();
    game.run();
}
