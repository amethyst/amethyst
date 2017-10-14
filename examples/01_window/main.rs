//! Opens an empty window.

extern crate amethyst;

use amethyst::ecs::rendering::{MaterialComponent, MeshComponent};
use amethyst::ecs::rendering::{create_render_system, RenderBundle};
use amethyst::ecs::transform::Transform;
use amethyst::event::{KeyboardInput, VirtualKeyCode};
use amethyst::prelude::*;
use amethyst::renderer::Config as DisplayConfig;
use amethyst::renderer::prelude::*;

struct Example;

impl State for Example {
    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } |
                WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}


type DrawFlat = pass::DrawFlat<PosNormTex, MeshComponent, MaterialComponent, Transform>;

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/01_window/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::new()),
    );

    let mut game = Application::build(Example)?
        .with_bundle(RenderBundle::new())?
        .with_local(create_render_system(pipe, Some(config))?)
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
