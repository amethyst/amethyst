//! Demonstrates loading prefabs using the Amethyst engine.

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    core::TransformBundle,
    prelude::*,
    renderer::{DrawShaded, PosNormTex},
    utils::{application_root_dir, scene::BasicScenePrefab},
    Error,
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct AssetsExample;

impl SimpleState for AssetsExample {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/example.ron", RonFormat, (), ())
        });
        data.world.create_entity().with(prefab_handle).build();
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    // Add our meshes directory to the asset loader.
    let resources_directory = app_root.join("examples/assets");

    let display_config_path = app_root.join("examples/prefab/resources/display_config.ron");

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;

    let mut game = Application::new(resources_directory, AssetsExample, game_data)?;
    game.run();
    Ok(())
}
