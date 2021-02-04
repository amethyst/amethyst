use amethyst::{assets::{DefaultLoader, Handle, Loader, LoaderBundle}, core::transform::TransformBundle, ecs::DispatcherBuilder, renderer::{types::DefaultBackend, RenderSkybox, RenderToWindow, RenderingBundle}, utils::application_root_dir, Application, GameData, SimpleState, StateData, SimpleTrans, Trans};
use amethyst::assets::prefab::Prefab;
use amethyst::renderer::{Camera, Mesh};
use amethyst::ui::UiTransform;
use amethyst::ecs::{Entity, IntoQuery};
use amethyst::renderer::light::Light;
use amethyst::core::Transform;
use amethyst::assets::{ProcessingQueue, AssetHandle};
use amethyst::renderer::rendy::mesh::{MeshBuilder, Position};
use amethyst::renderer::types::MeshData;
use amethyst::gltf::bundle::GltfBundle;

struct GltfExample;

#[derive(Debug, Clone, PartialEq)]
struct Scene {
}

impl SimpleState for GltfExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        let loader = resources.get::<DefaultLoader>().unwrap();
        let t: Handle<Prefab> = loader.load(
            "gltf/sample.gltf", // Here we load the associated ron file
        );
        world.push((t,));
    }
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {

        let StateData {
            world, resources, ..
        } = data;

        let mut q = <(Entity, &Camera)>::query();

        q.iter(*world).for_each(|e| println!("yeeeeeeaaaah {:?}", e));


        Trans::None
    }
}

fn main() -> Result<(), amethyst::Error> {
    let config = amethyst::LoggerConfig {
        level_filter: amethyst::LogLevelFilter::Debug,
        module_levels: vec![
            (
                "amethyst_assets".to_string(),
                amethyst::LogLevelFilter::Trace,
            ),
            ("amethyst_rendy".to_string(), amethyst::LogLevelFilter::Trace),
            ("distill_daemon".to_string(), amethyst::LogLevelFilter::Trace),
            ("distill_loader".to_string(), amethyst::LogLevelFilter::Trace),
            ("gfx_backend_metal::window".to_string(), amethyst::LogLevelFilter::Off),
        ],
        ..Default::default()
    };
    amethyst::start_logger(config);
    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets/");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher
        .add_bundle(LoaderBundle)
        //.add_bundle(GltfBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
                //.with_plugin(RenderPbr3D::default().with_skinning())
                .with_plugin(RenderSkybox::default()),
        );

    let game = Application::new(assets_dir, GltfExample, dispatcher)?;
    game.run();
    Ok(())
}
