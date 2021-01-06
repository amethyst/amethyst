use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    core::transform::TransformBundle,
    ecs::WorldExt,
    prelude::*,
    renderer::{
        plugins::{RenderPbr3D, RenderToWindow},
        rendy::{
            hal::command::ClearColor,
            mesh::{Normal, Position, Tangent, TexCoord},
        },
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
};

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>;

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/spotlights_scene.ron", RonFormat, ())
        });
        data.world.create_entity().with(handle).build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut game_data = DispatcherBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .add_bundle(TransformBundle::new())?
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderPbr3D::default()),
        )?;
    let game = Application::build(assets_dir, Example)?.build(game_data)?;
    game.run();
    Ok(())
}
