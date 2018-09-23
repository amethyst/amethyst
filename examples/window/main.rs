//! Opens an empty window.

extern crate amethyst;

use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosNormTex, RenderBundle, Stage};
use amethyst::utils::application_root_dir;

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let path = format!(
        "{}/examples/window/resources/display_config.ron",
        application_root_dir()
    );
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::<PosNormTex>::new()),
    );

    let game_data = GameDataBuilder::default().with_bundle(RenderBundle::new(pipe, Some(config)))?;
    let mut game = Application::new("./", Example, game_data)?;

    game.run();

    Ok(())
}
