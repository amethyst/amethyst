//! Opens an empty window.

extern crate amethyst;

use amethyst::{Application, Engine, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::config::Element;
use amethyst::gfx_device::DisplayConfig;

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        use amethyst::renderer::Layer;
        use amethyst::renderer::pass::Clear;

        let clear_layer = Layer::new("main", vec![Clear::new([0.0, 0.0, 0.0, 1.0])]);
        engine.pipe.layers = vec![clear_layer];
    }

    fn handle_events(&mut self, events: &[WindowEvent], _: &mut Engine) -> Trans {
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
    let cfg = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, cfg).done();
    game.run();
}
