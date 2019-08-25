//! Custom UI example

mod custom_pass;


use crate::custom_pass::RenderCustom;
use crate::custom_pass::Triangle;
use amethyst::{
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        Camera
    },
    core::transform::Transform,
    utils::application_root_dir,
};

pub struct Pong;

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;


        // Create a left plank entity.
        world
            .create_entity()
            .with(Triangle {
                points: [[0., 0.], [0., 1.], [1., 0.0]],
                colors: [[1., 0., 0., 1.], [0., 1., 0., 1.], [0., 0., 1., 1.]]
            })
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());



    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/custom_shader/config/display.ron");

    // This line is not mentioned in the pong tutorial as it is specific to the context
    // of the git repository. It only is a different location to load the assets from.
    let assets_dir = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default().with_bundle(
        RenderingBundle::<DefaultBackend>::new()
            // The RenderToWindow plugin provides all the scaffolding for opening a window and
            // drawing on it
            .with_plugin(
                RenderToWindow::from_config_path(display_config_path)
                    .with_clear([1.0, 1.0, 1.0, 1.0]),
            )
            // RenderFlat2D plugin is used to render entities with `SpriteRender` component.
            .with_plugin(RenderCustom::default()),

    )?;

    let mut game = Application::new(assets_dir, Pong, game_data)?;

    game.run();
    Ok(())
}
