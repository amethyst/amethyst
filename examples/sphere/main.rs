//! Displays a shaded sphere to the user.

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    core::transform::TransformBundle,
    ecs::prelude::{World, WorldExt},
    prelude::*,
    renderer::{
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::mesh::{Normal, Position, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
};

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, ())
        });
        data.world.create_entity().with(handle).build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("examples/sphere/config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let mut world = World::with_application_resources::<GameData<'_, '_>, _>(assets_dir)?;
    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::new(&mut world), "", &[])
        .with_bundle(
            &mut world,
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderShaded3D::default()),
        )?
        .with_bundle(&mut world, TransformBundle::new())?;
    let mut game = Application::new(Example, game_data, world)?;
    game.run();
    Ok(())
}
