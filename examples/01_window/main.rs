//! Opens an empty window.

extern crate amethyst;

use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::AssetManager;
use amethyst::config::Config;
use amethyst::ecs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::Pipeline;

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut World, _: &mut AssetManager, pipe: &mut Pipeline) {
        use amethyst::renderer::Layer;
        use amethyst::renderer::pass::Clear;

        let clear_layer = Layer::new("main", vec![Clear::new([0.0, 0.0, 0.0, 1.0])]);
        pipe.layers = vec![clear_layer];
    }

    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     _: &mut World,
                     _: &mut AssetManager,
                     _: &mut Pipeline)
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
    let path = format!("{}/examples/01_window/resources/config.ron",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::load(path);
    let mut game = Application::build(Example, cfg).done();
    game.run();
}
