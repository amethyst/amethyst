//! Displays spheres with physically based materials.

extern crate amethyst;
extern crate amethyst_assets;

use amethyst::{
    assets::AssetLoaderSystemData,
    core::{
        cgmath::{Deg, Vector3},
        Transform, TransformBundle,
    },
    prelude::*,
    renderer::*,
    utils::application_root_dir,
};

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();

        println!("Load mesh");
        let (mesh, albedo) = {
            let mesh = world.exec(|loader: AssetLoaderSystemData<Mesh>| {
                loader.load_from_data(
                    Shape::Sphere(32, 32).generate::<Vec<PosNormTangTex>>(None),
                    (),
                )
            });
            let albedo = world.exec(|loader: AssetLoaderSystemData<Texture>| {
                loader.load_from_data([1.0, 1.0, 1.0, 1.0].into(), ())
            });

            (mesh, albedo)
        };

        println!("Create spheres");
        for i in 0..5 {
            for j in 0..5 {
                let roughness = 1.0f32 * (i as f32 / 4.0f32);
                let metallic = 1.0f32 * (j as f32 / 4.0f32);

                let mut pos = Transform::default();
                pos.set_position(Vector3::new(
                    2.0f32 * (i - 2) as f32,
                    2.0f32 * (j - 2) as f32,
                    0.0,
                ));

                let metallic = [metallic, metallic, metallic, 1.0].into();
                let roughness = [roughness, roughness, roughness, 1.0].into();

                let (metallic, roughness) = world.exec(|loader: AssetLoaderSystemData<Texture>| {
                    (
                        loader.load_from_data(metallic, ()),
                        loader.load_from_data(roughness, ()),
                    )
                });

                let mtl = Material {
                    albedo: albedo.clone(),
                    metallic,
                    roughness,
                    ..mat_defaults.clone()
                };

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
            color: [0.8, 0.0, 0.0].into(),
            ..PointLight::default()
        }.into();

        let mut light1_transform = Transform::default();
        light1_transform.set_position(Vector3::new(6.0, 6.0, -6.0));

        let light2: Light = PointLight {
            intensity: 5.0,
            color: [0.0, 0.3, 0.7].into(),
            ..PointLight::default()
        }.into();

        let mut light2_transform = Transform::default();
        light2_transform.set_position(Vector3::new(6.0, -6.0, -6.0));

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
        transform.set_position(Vector3::new(0.0, 0.0, -12.0));
        transform.set_rotation(Deg(0.0), Deg(180.0), Deg(0.0));

        world
            .create_entity()
            .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
            .with(transform)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir();

    let path = format!(
        "{}/examples/material/resources/display_config.ron",
        app_root
    );

    let resources = format!("{}/examples/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(path, DrawPbm::<PosNormTangTex>::new(), false)?;
    let mut game = Application::new(&resources, Example, game_data)?;
    game.run();
    Ok(())
}
