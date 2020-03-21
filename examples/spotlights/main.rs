use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    core::transform::TransformBundle,
    ecs::WorldExt,
    prelude::*,
    renderer::{
        plugins::{RenderPbr3D, RenderToWindow},
        rendy::mesh::{Normal, Position, Tangent, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
    window::{DisplayConfig, EventLoop},
};
use amethyst_rendy::rendy;

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>;

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/spotlights_scene.ron", RonFormat, ())
        });
        data.world.create_entity().with(handle).build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/spotlights/config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let event_loop = EventLoop::new();
    let display_config = DisplayConfig::load(display_config_path)?;
    let game_data = GameDataBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)
                .with_plugin(
                    RenderToWindow::new().with_clear(rendy::hal::command::ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderPbr3D::default()),
        )?;
    let game = Application::new(assets_dir, Example, game_data)?;
    game.run_winit_loop(event_loop);
}
