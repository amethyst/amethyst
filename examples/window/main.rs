//! Opens an empty window.

extern crate amethyst;

use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat, Event, KeyboardInput, Pipeline, PosNormTex,
                         RenderBundle, RenderSystem, Stage, VirtualKeyCode, WindowEvent};

struct Example;

impl State for Example {
    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn run() -> Result<(), amethyst::Error> {
    let path = format!(
        "{}/examples/window/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);


    let mut renderer = None;
    let game = Application::build("./", Example)?
        .with_bundle(RenderBundle::new())?
        .world(|world| {
            let pipe = Pipeline::build().with_stage(
                Stage::with_backbuffer()
                    .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
                    .with_pass(DrawFlat::<PosNormTex>::new()),
            );
            renderer = Some(RenderSystem::build(world, pipe, Some(config)));
        })
        .with_local(renderer.unwrap()?);
    Ok(game.build()?.run())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}
