extern crate amethyst;

mod pong;
mod systems;

use amethyst::Result;
use amethyst::core::transform::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle, Stage};

fn run() -> Result<()> {
    use pong::Pong;

    let path = format!(
        "{}/examples/pong_tutorial_03/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let binding_path = format!(
        "{}/examples/pong_tutorial_03/resources/bindings_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );

    let input_bundle = InputBundle::<String, String>::new().with_bindings_from_file(binding_path);

    let mut game = Application::build("./", Pong)?
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .with_bundle(input_bundle)?
        .with(systems::PaddleSystem, "paddle_system", &["input_system"])
        .build()?;
    game.run();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Error occurred during game execution: {}", e);
        ::std::process::exit(1);
    }
}
