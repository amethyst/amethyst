extern crate amethyst;

mod pong;
mod paddle;

use amethyst::Result;
use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle, RenderSystem,
                         Stage};
use amethyst::core::transform::TransformBundle;
use amethyst::input::InputBundle;
use paddle::PaddleSystem;


fn run() -> Result<()> {
    use pong::Pong;

    let display_config = format!("{}/examples/pong_tutorial_03/resources/display_config.ron",
                                 env!("CARGO_MANIFEST_DIR"));
    let key_bindings_path = format!("{}/examples/pong_tutorial_03/resources/input.ron",
                                    env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&display_config);

    let pipe = Pipeline::build().with_stage(Stage::with_backbuffer()
        .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
        .with_pass(DrawFlat::<PosTex>::new()));

    let mut game = Application::build("./", Pong)?
        .with_bundle(
            InputBundle::<String, String>::new()
            .with_bindings_from_file(&key_bindings_path)
        )?
        .with::<PaddleSystem>(PaddleSystem, "paddle_system", &["input_system"])
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new())?
        .with_local(RenderSystem::build(pipe, Some(config))?)
        .build()?;

    Ok(game.run())
}

fn main() {
    if let Err(e) = run() {
        println!("Error occurred during game execution: {}", e);
        ::std::process::exit(1);
    }
}
