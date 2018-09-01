//! Displays a shaded sphere to the user.

extern crate amethyst;

use amethyst::assets::{Loader, PrefabLoader, PrefabLoaderSystem, ProgressCounter, RonFormat};
use amethyst::core::transform::GlobalTransform;
use amethyst::core::transform::TransformBundle;
use amethyst::prelude::*;
use amethyst::renderer::*;
use amethyst::utils::application_root_dir;
use amethyst::utils::scene::BasicScenePrefab;

type MyPrefabData = BasicScenePrefab<ComboMeshCreator>;

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let mut progress = ProgressCounter::default();

        let mesh: MeshHandle = {
            let mesh_storage = data.world.read_resource();
            let loader = data.world.read_resource::<Loader>();
            let vertices = vec![
                PosColorNorm {
                    position: [0.5, -0.5, 0.0],
                    color: [0.0, 0.3, 1.0, 1.0],
                    normal: [-0.2, 0.6, 0.0],
                },
                PosColorNorm {
                    position: [0.0, 0.5, 0.0],
                    color: [1.0, 0.1, 0.1, 1.0],
                    normal: [0.0, 0.3, 0.0],
                },
                PosColorNorm {
                    position: [-0.5, -0.5, 0.0],
                    color: [0.1, 0.9, 0.2, 1.0],
                    normal: [0.1, 1.0, 0.0],
                },
            ];
            loader.load_from_data(MeshData::from(vertices), &mut progress, &mesh_storage)
        };

        data.world
            .create_entity()
            .with(mesh)
            .with(GlobalTransform::new())
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = env!("CARGO_MANIFEST_DIR");

    let display_config_path = format!("{}/examples/debug_lines/resources/display.ron", app_root);

    let resources = format!("{}/examples/assets/", app_root);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.001, 0.005, 0.005, 1.0], 1.0)
            // .with_pass(DrawShadedSeparate::new()),
        .with_pass(DrawDebugLines::<PosColorNorm>::new()),
    );

    let config = DisplayConfig::load(display_config_path);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?;

    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}
