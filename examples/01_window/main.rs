//! Opens an empty window.

extern crate amethyst;
extern crate amethyst_renderer;
extern crate cgmath;

use amethyst::event::{KeyboardInput, VirtualKeyCode};
use amethyst::prelude::*;
use amethyst::renderer::prelude::{PosNormTex, Pipeline};
use amethyst::renderer::Config as DisplayConfig;

struct Example;

impl State for Example {
    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent {
                event, ..
            } => match event {
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape), ..
                    }, ..
                } | WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn run() -> Result<(), amethyst::Error> {
    let path = format!("{}/examples/01_window/resources/config.ron",
                       env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&path);

    let mut game = Application::build(Example)
        .with_renderer(Pipeline::forward::<PosNormTex>(), config)?
        .build()
        .expect("Fatal error");

    game.run();

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}
