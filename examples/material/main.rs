//! Displays spheres with physically based materials.
use amethyst::{
    assets::AssetLoaderSystemData,
    core::{
        ecs::{Builder, WorldExt},
        Transform, TransformBundle,
    },
    renderer::{
        camera::Camera,
        light::{Light, PointLight},
        mtl::{Material, MaterialDefaults},
        palette::{LinSrgba, Srgb},
        plugins::{RenderPbr3D, RenderToWindow},
        rendy::{
            mesh::{Normal, Position, Tangent, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        shape::Shape,
        types::DefaultBackend,
        Mesh, RenderingBundle, Texture,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    Application, GameData, GameDataBuilder, SimpleState, StateData,
};

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();

        println!("Load mesh");
        let (mesh, albedo) = {
            let mesh = world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
                loader.load_from_data(
                    Shape::Sphere(32, 32)
                        .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(None)
                        .into(),
                    (),
                )
            });
            let albedo = world.exec(|loader: AssetLoaderSystemData<'_, Texture>| {
                loader.load_from_data(
                    load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5)).into(),
                    (),
                )
            });

            (mesh, albedo)
        };

        println!("Create spheres");
        for i in 0..5 {
            for j in 0..5 {
                let roughness = 1.0f32 * (i as f32 / 4.0f32);
                let metallic = 1.0f32 * (j as f32 / 4.0f32);

                let mut pos = Transform::default();
                pos.set_translation_xyz(2.0f32 * (i - 2) as f32, 2.0f32 * (j - 2) as f32, 0.0);

                let mtl = world.exec(
                    |(mtl_loader, tex_loader): (
                        AssetLoaderSystemData<'_, Material>,
                        AssetLoaderSystemData<'_, Texture>,
                    )| {
                        let metallic_roughness = tex_loader.load_from_data(
                            load_from_linear_rgba(LinSrgba::new(0.0, roughness, metallic, 0.0))
                                .into(),
                            (),
                        );

                        mtl_loader.load_from_data(
                            Material {
                                albedo: albedo.clone(),
                                metallic_roughness,
                                ..mat_defaults.clone()
                            },
                            (),
                        )
                    },
                );

                world
                    .create_entity()
                    .with(pos)
                    .with(mesh.clone())
                    .with(mtl)
                    .build();
            }
        }

        println!("Create lights");
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

        world
            .create_entity()
            .with(light1)
            .with(light1_transform)
            .build();

        world
            .create_entity()
            .with(light2)
            .with(light2_transform)
            .build();

        println!("Put camera");

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -12.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        world
            .create_entity()
            .with(Camera::standard_3d(width, height))
            .with(transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("examples/material/config/display.ron");
    let assets_dir = app_root.join("examples/assets/");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderPbr3D::default()),
        )?;

    let mut game = Application::new(assets_dir, Example, game_data)?;
    game.run();
    Ok(())
}
