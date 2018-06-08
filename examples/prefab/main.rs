//! Demonstrates loading prefabs using the Amethyst engine.

extern crate amethyst;
extern crate rayon;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::config::Config;
use amethyst::core::transform::{Transform, TransformBundle};
use amethyst::input::{is_close_requested, is_key, InputBundle};
use amethyst::renderer::{CameraPrefab, DisplayConfig, DrawShaded, Event, GraphicsPrefab, Light,
                         ObjFormat, Pipeline, PosNormTex, RenderBundle, Stage, TextureFormat,
                         VirtualKeyCode};
use amethyst::{Application, Error, GameData, GameDataBuilder, State, StateData, Trans};

type MyPrefabData = (
    Option<GraphicsPrefab<Vec<PosNormTex>>>,
    Option<Transform>,
    Option<Light>,
    Option<CameraPrefab>,
);

struct AssetsExample;

impl<'a, 'b> State<GameData<'a, 'b>> for AssetsExample {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        let prefab_handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/example.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_key(&event, VirtualKeyCode::Escape) || is_close_requested(&event) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

/// Wrapper around the main, so we can return errors easily.
fn main() -> Result<(), Error> {
    // Add our meshes directory to the asset loader.
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/asset_loading/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?;

    let mut game = Application::new(resources_directory, AssetsExample, game_data)?;
    game.run();
    Ok(())
}
