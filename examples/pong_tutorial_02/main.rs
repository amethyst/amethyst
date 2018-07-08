extern crate amethyst;

mod pong;

use amethyst::core::transform::TransformBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DrawFlat, PosTex};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    use pong::Pong;

    let path = format!(
        "{}/examples/pong_tutorial_02/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(path, DrawFlat::<PosTex>::new(), false)?;
    let mut game = Application::new("./", Pong, game_data)?;
    game.run();
    Ok(())
}
