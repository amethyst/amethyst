//! Pong Tutorial 1

use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};

pub struct Pong;

impl SimpleState for Pong {}

const BG_COLOR: [f32; 4] = [0.34, 0.36, 0.52, 1.0];

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let config_dir = app_root.join("examples/pong_tutorial_01/config/");
    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = app_root.join("examples/assets/");

    let display_config_path = config_dir.join("display.ron");

    let render_bundle = RenderingBundle::<DefaultBackend>::new()
        // The RenderToWindow plugin provides all the scaffolding for opening a window and
        // drawing on it
        .with_plugin(RenderToWindow::from_config_path(display_config_path)?.with_clear(BG_COLOR))
        .with_plugin(RenderFlat2D::default());

    let game_data = GameDataBuilder::default().with_bundle(render_bundle)?;

    let mut game = Application::new(assets_dir, Pong, game_data)?;
    game.run();

    Ok(())
}
