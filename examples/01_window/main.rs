//! Opens an empty window.

extern crate amethyst;

use amethyst::prelude::*;
use amethyst::ecs::systems::TransformSystem;

struct Example;

impl State for Example {
    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        if let Event::Window(e) = event {
            match e {
                WindowEvent::KeyboardInput(_, _, Some(VirtualKeyCode::Escape), _) |
                WindowEvent::Closed => return Trans::Quit,
                _ => (),
            }
        }
    }
}

fn main() {
    let path = format!("{}/examples/01_window/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));

    let cfg = Config::from_file(path).unwrap();
    let mut game = Application::build(Example, cfg)
        .with_system::<TransformSystem>("trans", 0)
        .finish()
        .expect("Fatal error");

    game.run();
}
