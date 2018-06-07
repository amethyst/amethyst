//! Displays spheres with physically based materials.

extern crate amethyst;

use amethyst::assets::Loader;
use amethyst::core::cgmath::{Deg, Matrix4};
use amethyst::core::transform::GlobalTransform;
use amethyst::input::{is_close_requested, is_key};
use amethyst::prelude::*;
use amethyst::renderer::*;

struct Example;

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();
        let verts = Shape::Sphere(32, 32).generate::<Vec<PosNormTangTex>>(None);
        let albedo = [1.0, 1.0, 1.0, 1.0].into();

        println!("Load mesh");
        let (mesh, albedo) = {
            let loader = world.read_resource::<Loader>();

            let meshes = &world.read_resource();
            let textures = &world.read_resource();
            let mesh: MeshHandle = loader.load_from_data(verts, (), meshes);
            let albedo = loader.load_from_data(albedo, (), textures);

            (mesh, albedo)
        };

        println!("Create spheres");
        for i in 0..5 {
            for j in 0..5 {
                let roughness = 1.0f32 * (i as f32 / 4.0f32);
                let metallic = 1.0f32 * (j as f32 / 4.0f32);
                let pos = Matrix4::from_translation(
                    [2.0f32 * (i - 2) as f32, 2.0f32 * (j - 2) as f32, 0.0].into(),
                );

                let metallic = [metallic, metallic, metallic, 1.0].into();
                let roughness = [roughness, roughness, roughness, 1.0].into();

                let (metallic, roughness) = {
                    let loader = world.read_resource::<Loader>();
                    let textures = &world.read_resource();

                    let metallic = loader.load_from_data(metallic, (), textures);
                    let roughness = loader.load_from_data(roughness, (), textures);

                    (metallic, roughness)
                };

                let mtl = Material {
                    albedo: albedo.clone(),
                    metallic,
                    roughness,
                    ..mat_defaults.clone()
                };

                world
                    .create_entity()
                    .with(GlobalTransform(pos.into()))
                    .with(mesh.clone())
                    .with(mtl)
                    .build();
            }
        }

        println!("Create lights");
        let light1: Light = PointLight {
            center: [6.0, 6.0, -6.0].into(),
            intensity: 6.0,
            color: [0.8, 0.0, 0.0].into(),
            ..PointLight::default()
        }.into();

        let light2: Light = PointLight {
            center: [6.0, -6.0, -6.0].into(),
            intensity: 5.0,
            color: [0.0, 0.3, 0.7].into(),
            ..PointLight::default()
        }.into();

        world.create_entity().with(light1).build();

        world.create_entity().with(light2).build();

        println!("Put camera");

        let transform =
            Matrix4::from_translation([0.0, 0.0, -12.0].into()) * Matrix4::from_angle_y(Deg(180.));
        world
            .create_entity()
            .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
            .with(GlobalTransform(transform.into()))
            .build();
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn main() -> amethyst::Result<()> {
    let path = format!(
        "{}/examples/material/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&path);

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawPbm::<PosNormTangTex>::new()),
    );
    let game_data = GameDataBuilder::default().with_bundle(RenderBundle::new(pipe, Some(config)))?;
    let mut game = Application::new(&resources, Example, game_data)?;
    game.run();
    Ok(())
}
