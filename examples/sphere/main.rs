//! Displays a shaded sphere to the user.

extern crate amethyst;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat};
use amethyst::core::transform::{Transform, TransformBundle};
use amethyst::input::{is_close_requested, is_key};
use amethyst::prelude::*;
use amethyst::renderer::{CameraPrefab, DrawShaded, Event, GraphicsPrefab, LightPrefab, PosNormTex,
                         VirtualKeyCode};

type MyPrefabData = (
    Option<GraphicsPrefab<Vec<PosNormTex>>>,
    Option<Transform>,
    Option<LightPrefab>,
    Option<CameraPrefab>,
);

struct Example;

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        // Initialise the scene with an object, a light and a camera.
        let handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, (), ())
        });
        data.world.create_entity().with(handle).build();
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key(&event, VirtualKeyCode::Escape) {
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

fn main() -> amethyst::Result<()> {
    let display_config_path = format!(
        "{}/examples/sphere/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;
    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}
