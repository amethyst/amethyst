//! Pong Tutorial 1

use amethyst::{
    assets::LoaderBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};

pub struct Pong;

impl SimpleState for Pong {}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");

    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = app_root.join("assets/");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        // The LoaderBundle manages asset loading and storage.
        .add_bundle(LoaderBundle)
        // The RenderingBundle provides a mechanism for registering rendering plugins.
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides functionality to open a window and draw on it.
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    }),
                )
                // The RenderFlat2D plugin renders entities with a `SpriteRender` component.
                .with_plugin(RenderFlat2D::default()),
        );

    let game = Application::new(assets_dir, Pong, dispatcher)?;
    game.run();
    Ok(())
}
