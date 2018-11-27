//! Demonstrates loading prefabs using the Amethyst engine.

extern crate amethyst;
extern crate rayon;

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

impl<S, E> StateCallback<S, E> for AssetsExample {
    fn on_start(&mut self, world: &mut World) {
        let prefab_handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/example.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    // Add our meshes directory to the asset loader.
    let resources_directory = format!("{}/examples/assets", app_root);

    let display_config_path = format!("{}/examples/prefab/resources/display_config.ron", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;

    let mut game = Application::build(resources_directory)?
        .with_state((), AssetsExample)?
        .build(game_data)?;

    game.run();
    Ok(())
}
