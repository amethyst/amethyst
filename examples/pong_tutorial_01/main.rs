extern crate amethyst;

use amethyst::prelude::*;
use amethyst::renderer::{
    DisplayConfig, DrawFlat, Event, KeyboardInput, Pipeline, PosTex, RenderBundle, Stage,
    VirtualKeyCode, WindowEvent,
};
use amethyst::Result;

struct Pong;

impl State for Pong {
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

fn main() -> Result<()> {
    let path = format!(
        "{}/examples/pong_tutorial_01/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );

    let mut game = Application::build("./", Pong)?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .build()?;
    game.run();
    Ok(())
}
