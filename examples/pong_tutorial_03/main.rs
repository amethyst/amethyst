extern crate amethyst;

mod pong;
mod systems;

use amethyst::core::transform::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::DrawSprite;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    use pong::Pong;

    let path = format!(
        "{}/examples/pong_tutorial_03/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let binding_path = format!(
        "{}/examples/pong_tutorial_03/resources/bindings_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let input_bundle =
        InputBundle::<String, String>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(path, DrawSprite::new(), false)?
        .with_bundle(input_bundle)?
        .with(systems::PaddleSystem, "paddle_system", &["input_system"]);
    let mut game = Application::new(assets_dir, Pong, game_data)?;
    game.run();
    Ok(())
}
