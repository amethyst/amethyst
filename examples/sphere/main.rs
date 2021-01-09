//! Displays a shaded sphere to the user.

use amethyst::{
    assets::{DefaultLoader, Handle, Loader, LoaderBundle, ProcessingQueue},
    core::transform::{Transform, TransformBundle},
    prelude::*,
    renderer::{
        light::{Light, PointLight},
        loaders::load_from_linear_rgba,
        palette::{LinSrgba, Srgb},
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::{
            hal::command::ClearColor,
            mesh::{Normal, Position, Tangent, TexCoord},
        },
        shape::Shape,
        types::{DefaultBackend, MeshData, TextureData},
        Camera, Material, MaterialDefaults, Mesh, RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
};

struct SphereExample;

impl SimpleState for SphereExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let loader = data.resources.get::<DefaultLoader>().unwrap();
        let mesh_storage = data.resources.get::<ProcessingQueue<MeshData>>().unwrap();
        let tex_storage = data
            .resources
            .get::<ProcessingQueue<TextureData>>()
            .unwrap();
        let mtl_storage = data.resources.get::<ProcessingQueue<Material>>().unwrap();

        let mesh: Handle<Mesh> = loader.load_from_data(
            Shape::Sphere(64, 64)
                .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(None)
                .into(),
            (),
            &mesh_storage,
        );

        let albedo = loader.load_from_data(
            load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5)).into(),
            (),
            &tex_storage,
        );

        let mtl: Handle<Material> = {
            let mat_defaults = data.resources.get::<MaterialDefaults>().unwrap().0.clone();

            loader.load_from_data(
                Material {
                    albedo,
                    ..mat_defaults
                },
                (),
                &mtl_storage,
            )
        };

        data.world.push((Transform::default(), mesh, mtl));

        let light1: Light = PointLight {
            intensity: 6.0,
            color: Srgb::new(0.8, 0.0, 0.0),
            ..PointLight::default()
        }
        .into();

        let mut light1_transform = Transform::default();
        light1_transform.set_translation_xyz(6.0, 6.0, -6.0);

        let light2: Light = PointLight {
            intensity: 5.0,
            color: Srgb::new(0.0, 0.3, 0.7),
            ..PointLight::default()
        }
        .into();

        let mut light2_transform = Transform::default();
        light2_transform.set_translation_xyz(6.0, -6.0, -6.0);

        data.world
            .extend(vec![(light1, light1_transform), (light2, light2_transform)]);

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -4.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        let (width, height) = {
            let dim = data.resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
        };

        data.world
            .extend(vec![(Camera::standard_3d(width, height), transform)]);
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(amethyst::LoggerConfig {
        level_filter: log::LevelFilter::Debug,
        ..Default::default()
    })
    .start();

    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("config/display.ron");
    let assets_dir = app_root.join("assets");

    let mut game_data = DispatcherBuilder::default();
    game_data
        .add_bundle(LoaderBundle)
        .add_bundle(TransformBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                        float32: [0.34, 0.36, 0.52, 1.0],
                    }),
                )
                .with_plugin(RenderShaded3D::default()),
        );
    let game = Application::build(assets_dir, SphereExample)?.build(game_data)?;
    game.run();
    Ok(())
}
