//! Demonstrates how to use the fly camera

extern crate amethyst;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::controls::FlyControlBundle;
use amethyst::core::transform::TransformBundle;
use amethyst::core::WithNamed;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DrawShaded, PosNormTex};
use amethyst::utils::scene::BasicScenePrefab;
use amethyst::Error;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct ExampleState;

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
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

    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/fly_camera/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let key_bindings_path = format!(
        "{}/examples/fly_camera/resources/input.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(
            FlyControlBundle::<String, String>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            ).with_sensitivity(0.1, 0.1),
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
