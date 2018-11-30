extern crate amethyst;

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    core::transform::TransformBundle,
    prelude::*,
    renderer::{DrawPbm, PosNormTangTex},
    utils::{application_root_dir, scene::BasicScenePrefab},
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTangTex>>;

struct Example;

impl<S, E> StateCallback<S, E> for Example {
    fn on_start(&mut self, world: &mut World) {
        let handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/spotlights_scene.ron", RonFormat, (), ())
        });
        world.create_entity().with(handle).build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let display_config_path = format!(
        "{}/examples/spotlights/resources/display_config.ron",
        app_root
    );

    let resources = format!("{}/examples/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawPbm::<PosNormTangTex>::new(), false)?;

    let mut game = Application::build(resources)?
        .with_state((), Example)?
        .build(game_data)?;

    game.run();
    Ok(())
}
