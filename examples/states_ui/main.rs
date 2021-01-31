use amethyst::{
    audio::AudioBundle,
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{
        plugins::RenderToWindow, rendy::hal::command::ClearColor, types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::{application_root_dir, fps_counter::FpsCounterBundle},
};

mod credits;
mod events;
mod game;
mod menu;
mod pause;
mod welcome;

/// Quick overview what you can do when running this example:
///
/// Switch from the 'Welcome' Screen to the 'Menu' Screen.
/// From the 'Menu', switch to either 'Credits' (from which you can only switch back) or to 'Game'.
/// From 'Game', you can enter the 'Pause' menu.
/// Here you can select to resume (go back to 'Game'), exit to menu (go to 'Menu') or exit (quit).
///
/// During the 'Pause' menu, the 'Game' is paused accordingly.

pub fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    // this will be the directory the 'Cargo.toml' is defined in.
    let app_root = application_root_dir()?;

    // our display config is in our configs folder.
    let display_config_path = app_root.join("config/display.ron");

    // other assets ('*.ron' files, '*.png' textures, '*.ogg' audio files, ui prefab files, ...) are here
    let assets_dir = app_root.join("assets");

    let mut game_data = DispatcherBuilder::default()
        // a lot of other bundles/systems depend on this (without it being explicitly clear), so it
        // makes sense to add it early on
        .add_bundle(TransformBundle::new())?
        // This system is in 'events.rs'. Basically, it registers UI events that
        // happen. Without it, the buttons will not react.
        .add_bundle(InputBundle::new())?
        // this bundle allows us to 'find' the Buttons and other UI elements later on
        .add_bundle(UiBundle::new())?
        // without this Bundle, our Program will silently (!) fail when trying to start the 'Game'.
        // (try it!)
        // It takes care of Audio (in this case, the Button audio for hovering/clicking)
        .add_bundle(AudioBundle::default())?
        // With this System, we can register UI events and act accordingly.
        // In this example it simply prints the events, excluding it does not provide less
        // functionality.
        .with_system_desc(
            crate::events::UiEventHandlerSystemDesc::default(),
            "ui_event_handler",
            &[],
        )
        // Necessary for the FPS counter in the upper left corner to work.
        // (simply uncommenting will fail at runtime, since the resource is expected to exist, you
        // need to uncomment line 107-114 in game.rs for it to still work)
        .add_bundle(FpsCounterBundle)?
        // Without this, we would not get a picture.
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // This creates the window and draws a background, if we don't specify a
                // background in the loaded ui prefab file.
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.005, 0.005, 0.005, 1.0],
                    }),
                )
                // Without this, all of our beautiful UI would not get drawn.
                // It will work, but we won't see a thing.
                .with_plugin(RenderUi::default()),
            /* If you want to draw Sprites and such, you would need this additionally:
             * .with_plugin(RenderFlat2D::default()) */
        )?;

    // creating the Application with the assets_dir, the first Screen, and the game_data with it's
    // systems.
    let game = Application::build(
        assets_dir,
        crate::welcome::WelcomeScreen::default(),
        game_data,
    )?;
    log::info!("Starting with WelcomeScreen!");
    game.run();

    Ok(())
}
