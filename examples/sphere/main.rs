//! Displays a shaded sphere to the user.

use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    renderer::{
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::mesh::{Normal, Position, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};
use amethyst_assets::{AssetStorage, Loader};
use amethyst_core::transform::Transform;
use amethyst_rendy::{
    light::{Light, PointLight},
    loaders::load_from_linear_rgba,
    rendy::mesh::Tangent,
    shape::Shape,
    Camera, Material, MaterialDefaults, Mesh, Texture,
};
use amethyst_window::ScreenDimensions;
use palette::{LinSrgba, Srgb};

struct SphereExample;

impl SimpleState for SphereExample {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let loader = data.resources.get::<Loader>().unwrap();
        let mesh_storage = data.resources.get::<AssetStorage<Mesh>>().unwrap();
        let tex_storage = data.resources.get::<AssetStorage<Texture>>().unwrap();
        let mtl_storage = data.resources.get::<AssetStorage<Material>>().unwrap();

        let mesh = loader.load_from_data(
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

        let mtl = {
            let mat_defaults = data.resources.get::<MaterialDefaults>().unwrap().0.clone();

            loader.load_from_data(
                Material {
                    albedo: albedo.clone(),
                    ..mat_defaults.clone()
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
    let app_root = application_root_dir()?;

    let display_config_path = app_root.join("examples/sphere/config/display.ron");
    let assets_dir = app_root.join("examples/sphere/assets");

    let mut game_data = DispatcherBuilder::default();
    game_data.add_bundle(TransformBundle).add_bundle(
        RenderingBundle::<DefaultBackend>::new()
            .with_plugin(
                RenderToWindow::from_config_path(display_config_path)?
                    .with_clear([0.34, 0.36, 0.52, 1.0]),
            )
            .with_plugin(RenderShaded3D::default()),
    );
    let mut game = Application::build(assets_dir, SphereExample)?.build(game_data)?;
    game.run();
    Ok(())
}
