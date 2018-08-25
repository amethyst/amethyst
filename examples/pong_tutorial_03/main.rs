extern crate amethyst;

mod pong;
mod systems;

use amethyst::core::transform::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawSprite, Pipeline, RenderBundle, Stage};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    use pong::Pong;

    let path = format!(
        "{}/examples/pong_tutorial_03/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawSprite::new()),
    );

    let binding_path = format!(
        "{}/examples/pong_tutorial_03/resources/bindings_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let input_bundle = InputBundle::<String, String>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with(systems::PaddleSystem, "paddle_system", &["input_system"]);
    let mut game = Application::new(assets_dir, Pong, game_data)?;
    game.run();
    Ok(())
}
