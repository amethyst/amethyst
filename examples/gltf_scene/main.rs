use amethyst::{
    assets::{
        prefab::Prefab, AssetHandle, DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue,
    },
    core::{math::Vector3, transform::TransformBundle, Transform},
    ecs::{DispatcherBuilder, Entity, IntoQuery},
    gltf::bundle::GltfBundle,
    renderer::{
        light::Light,
        rendy::mesh::{MeshBuilder, Position},
        types::{DefaultBackend, MeshData},
        Camera, Material, Mesh, RenderFlat3D, RenderPbr3D, RenderSkybox, RenderToWindow,
        RenderingBundle,
    },
    ui::UiTransform,
    utils::application_root_dir,
    Application, GameData, SimpleState, SimpleTrans, StateData, Trans,
};

struct GltfExample;

#[derive(Debug, Clone, PartialEq)]
struct Scene {}

impl SimpleState for GltfExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        let loader = resources.get::<DefaultLoader>().unwrap();
        let t: Handle<Prefab> = loader.load("gltf/sample.glb");
        world.push((t,));
    }
}

fn main() -> Result<(), amethyst::Error> {
    let config = amethyst::LoggerConfig {
        level_filter: amethyst::LogLevelFilter::Warn,
        module_levels: vec![],
        ..Default::default()
    };
    amethyst::start_logger(config);
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        .add_bundle(LoaderBundle)
        .add_bundle(GltfBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
                .with_plugin(RenderPbr3D::default())
                .with_plugin(RenderSkybox::default()),
        );

    let game = Application::new(assets_dir, GltfExample, dispatcher)?;
    game.run();
    Ok(())
}
