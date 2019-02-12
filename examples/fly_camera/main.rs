//! Demonstrates how to use the fly camera

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    controls::FlyControlBundle,
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{DrawShaded, PosNormTex},
    utils::{application_root_dir, scene::BasicScenePrefab},
    Error,
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/fly_camera.ron", RonFormat, (), ())
        });
        data.world
            .create_entity()
            .named("Fly Camera Scene")
            .with(prefab_handle)
            .build();
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources_directory = app_root.join("examples/assets");

    let display_config_path = app_root.join("examples/fly_camera/resources/display_config.ron");

    let key_bindings_path = app_root.join("examples/fly_camera/resources/input.ron");

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(
            FlyControlBundle::<String, String>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            )
            .with_sensitivity(0.1, 0.1),
        )?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;
    let mut game = Application::build(resources_directory, ExampleState)?.build(game_data)?;
    game.run();
    Ok(())
}
