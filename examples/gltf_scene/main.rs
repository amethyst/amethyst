use amethyst::utils::application_root_dir;
use amethyst::ecs::DispatcherBuilder;
use amethyst::{Application, SimpleState, StateData, GameData};
use amethyst::core::transform::TransformBundle;
use amethyst::renderer::{RenderingBundle, RenderToWindow, RenderSkybox, RenderPbr3D};
use amethyst::renderer::types::DefaultBackend;
use amethyst::assets::{LoaderBundle, DefaultLoader, Handle, Loader};
use amethyst::gltf::GltfAsset;

struct GltfExample;

#[derive(Debug, Clone, PartialEq)]
struct Scene{
    gltf_handle: Handle<GltfAsset>
}

impl SimpleState for GltfExample{
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData { world, resources, .. } = data;
        let loader = resources.get::<DefaultLoader>().unwrap();
        let res: Handle<GltfAsset> = loader.load(
            "gltf/sample.gltf", // Here we load the associated ron file
        );
        println!("res {:?}", res);
        world.push((res,));
    }
}

fn main() -> Result<(), amethyst::Error> {
    amethyst::start_logger(Default::default());
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