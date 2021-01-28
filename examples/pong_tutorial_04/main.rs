//! Pong Tutorial 4

mod pong;
mod systems;

use amethyst::{
    assets::LoaderBundle,
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};
use systems::{bounce::BounceSystem, move_balls::BallSystem};

use crate::{pong::Pong, systems::paddle::PaddleSystem};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");

    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = app_root.join("assets/");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        .add_bundle(LoaderBundle)
        // Add the transform bundle which handles tracking entity positions
        .add_bundle(TransformBundle)
        .add_bundle(
            InputBundle::new().with_bindings_from_file(app_root.join("config/bindings.ron"))?,
        )
        // We have now added our own systems, defined in the systems module
        .add_system(PaddleSystem)
        .add_system(BallSystem)
        .add_system(BounceSystem)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides all the scaffolding for opening a window and
                // drawing on it
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    }),
                )
                // RenderFlat2D plugin is used to render entities with `SpriteRender` component.
                .with_plugin(RenderFlat2D::default()),
        );

    let game = Application::new(assets_dir, Pong::default(), dispatcher)?;
    game.run();
    Ok(())
}
