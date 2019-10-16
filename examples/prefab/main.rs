//! Demonstrates loading prefabs using the Amethyst engine.

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    core::TransformBundle,
    prelude::*,
    renderer::{
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::mesh::{Normal, Position, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
    Error,
};

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

struct AssetsExample;

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/example.ron", RonFormat, ())
        });
        data.world.create_entity().with(prefab_handle).build();
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let assets_dir = app_root.join("examples/assets");

    let display_config_path = app_root.join("examples/prefab/config/display.ron");

    let game_data = GameDataBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderShaded3D::default()),
        )?;

    let mut game = Application::new(assets_dir, AssetsExample, game_data)?;
    game.run();
    Ok(())
}
