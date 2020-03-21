//! Pong Tutorial 3

mod pong;
mod systems;

use crate::pong::Pong;
use amethyst::{
    core::TransformBundle,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
    window::{DisplayConfig, EventLoop},
};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/pong_tutorial_03/config/display.ron");

    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = app_root.join("examples/assets/");

    let event_loop = EventLoop::new();
    let display_config = DisplayConfig::load(display_config_path)?;
    let game_data = GameDataBuilder::default()
        // Add the transform bundle which handles tracking entity positions
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(
                app_root.join("examples/pong_tutorial_03/config/bindings.ron"),
            )?,
        )?
        // We have now added our own system, the PaddleSystem, defined in systems/paddle.rs
        .with(systems::PaddleSystem, "paddle_system", &["input_system"])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)
                // The RenderToWindow plugin provides all the scaffolding for opening a window and
                // drawing on it
                .with_plugin(RenderToWindow::new().with_clear(ClearColor {
                    float32: [0.0, 0.0, 0.0, 1.0],
                }))
                // RenderFlat2D plugin is used to render entities with `SpriteRender` component.
                .with_plugin(RenderFlat2D::default()),
        )?;

    let game = Application::new(assets_dir, Pong, game_data)?;
    game.run_winit_loop(event_loop);
}
