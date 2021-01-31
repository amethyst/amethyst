use amethyst::{assets::{DefaultLoader, Handle, Loader, LoaderBundle}, core::transform::TransformBundle, ecs::DispatcherBuilder, gltf::GltfAsset, renderer::{types::DefaultBackend, RenderSkybox, RenderToWindow, RenderingBundle}, utils::application_root_dir, Application, GameData, SimpleState, StateData, SimpleTrans, Trans};
use amethyst::assets::prefab::Prefab;
use amethyst::renderer::Camera;
use amethyst::ui::UiTransform;
use amethyst::ecs::{Entity, IntoQuery};

struct GltfExample;

#[derive(Debug, Clone, PartialEq)]
struct Scene {
    gltf_handle: Handle<GltfAsset>,
}

impl SimpleState for GltfExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;
        let loader = resources.get::<DefaultLoader>().unwrap();
        let res: Handle<Camera> = loader.load(
            "gltf/sample.gltf", // Here we load the associated ron file
        );
        let entity = world.push((res,));
        println!("entity ? {:?}", entity);
    }
    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {

        let StateData {
            world, resources, ..
        } = data;
        let mut q = <(&Camera)>::query();

        q.iter(*world).for_each(|e| println!("yeeeeeeaaaah"));


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
            ("atelier_daemon".to_string(), amethyst::LogLevelFilter::Trace),
            ("atelier_loader".to_string(), amethyst::LogLevelFilter::Trace),
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
