//! Opens an empty window.

extern crate amethyst;
extern crate genmesh;
extern crate cgmath;

use amethyst::engine::{Application, State, Trans};
use amethyst::config::Element;
use amethyst::ecs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::asset_manager::AssetManager;
use amethyst::components::event::EngineEvent;
use amethyst::renderer::Pipeline;

struct Example;

impl State for Example {
    fn on_start(&mut self, _: &mut World, _: &mut AssetManager, pipeline: &mut Pipeline) {
        use amethyst::renderer::pass::Clear;
        use amethyst::renderer::Layer;
        let clear_layer =
            Layer::new("main",
                        vec![
                            Clear::new([0.0, 0.0, 0.0, 1.0]),
                        ]);
        pipeline.layers = vec![clear_layer];
    }

    fn handle_events(&mut self, events: &[EngineEvent], _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        use amethyst::components::event::*;
        for event in events {
            match event.payload {
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
    let display_config = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, display_config).done();
    game.run();
}
