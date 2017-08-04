//! Opens an empty window.

extern crate amethyst;

use amethyst::prelude::*;
use amethyst::ecs::systems::{RenderSystem, SystemExt};
use amethyst::ecs::resources::{KeyboardInput, KeyCode};

struct Example;

impl State for Example {
    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::Window(e) => match e {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(KeyCode::Escape), ..
                    }, ..
                } | WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn main() {
    let path = format!("{}/examples/01_window/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));

    let mut game = Application::build(Example)
        .with_thread_local(RenderSystem::build(()).unwrap())
        .build()
        .expect("Fatal error");

    game.run();
}
