//! Displays a shaded sphere to the user, using multisampling.

extern crate amethyst;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::core::transform::TransformBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DrawShaded, PosNormTex};
use amethyst::utils::application_root_dir;
use amethyst::utils::scene::BasicScenePrefab;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        // Initialise the scene with an object, a light and a camera.
        let handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, (), ())
        });
        data.world.create_entity().with(handle).build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let display_config_path = format!(
        "{}/examples/sphere_multisample/resources/display_config.ron",
        app_root
    );

    let resources = format!("{}/examples/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;
    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}
