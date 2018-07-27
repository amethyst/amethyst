//! Demonstrates loading custom assets using the Amethyst engine.
// TODO: Add asset loader directory store for the meshes.

extern crate amethyst;
extern crate rayon;

use amethyst::assets::{Loader, Result as AssetResult, SimpleFormat};
use amethyst::core::cgmath::{Array, Matrix4, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::renderer::{
    Camera, DrawShaded, Event, Light, Material, MaterialDefaults, Mesh, MeshData, PointLight,
    PosNormTex, Projection, Rgba, VirtualKeyCode,
};
use amethyst::Error;

#[derive(Clone)]
struct Custom;

impl SimpleFormat<Mesh> for Custom {
    const NAME: &'static str = "CUSTOM";

    type Options = ();

    /// Reads the given bytes and produces asset data.
    fn import(&self, bytes: Vec<u8>, _: ()) -> AssetResult<MeshData> {
        let data: String = String::from_utf8(bytes)?;

        let trimmed: Vec<&str> = data.lines().filter(|line| line.len() >= 1).collect();

        let mut result = Vec::new();

        for line in trimmed {
            let nums: Vec<&str> = line.split_whitespace().collect();

            let position = [
                nums[0].parse::<f32>().unwrap(),
                nums[1].parse::<f32>().unwrap(),
                nums[2].parse::<f32>().unwrap(),
            ];

            let normal = [
                nums[3].parse::<f32>().unwrap(),
                nums[4].parse::<f32>().unwrap(),
                nums[5].parse::<f32>().unwrap(),
            ];

            result.push(PosNormTex {
                position,
                normal,
                tex_coord: [0.0, 0.0],
            });
        }
        Ok(result.into())
    }
}

struct AssetsExample;

impl<'a, 'b> State<GameData<'a, 'b>> for AssetsExample {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        world.add_resource(0usize);

        initialise_camera(world);
        initialise_lights(world);

        // Add custom cube object to scene
        let (mesh, mtl) = {
            let mat_defaults = world.read_resource::<MaterialDefaults>();
            let loader = world.read_resource::<Loader>();

            let meshes = &world.read_resource();
            let textures = &world.read_resource();

            let mesh = loader.load("mesh/cuboid.custom", Custom, (), (), meshes);
            let albedo = loader.load_from_data([0.0, 0.0, 1.0, 0.0].into(), (), textures);
            let mat = Material {
                albedo,
                ..mat_defaults.0.clone()
            };

            (mesh, mat)
        };

        let mut trans = Transform::default();
        trans.translation = Vector3::new(-5.0, 0.0, 0.0);
        trans.scale = Vector3::from_value(2.);
        world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(GlobalTransform::default())
            .build();
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
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

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    // Add our meshes directory to the asset loader.
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/asset_loading/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(TransformBundle::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;

    let mut game = Application::new(resources_directory, AssetsExample, game_data)?;
    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    use amethyst::core::cgmath::{Deg, Matrix4};
    let transform =
        Matrix4::from_translation([0., -20., 10.].into()) * Matrix4::from_angle_x(Deg(75.96));
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.0, Deg(60.0))))
        .with(GlobalTransform(transform.into()))
        .build();
}

/// Adds lights to the scene.
fn initialise_lights(world: &mut World) {
    let light: Light = PointLight {
        intensity: 100.0,
        radius: 1.0,
        color: Rgba::white(),
        ..Default::default()
    }.into();

    let transform = Matrix4::from_translation([5.0, -20.0, 15.0].into());

    // Add point light.
    world
        .create_entity()
        .with(light)
        .with(GlobalTransform(transform.into()))
        .build();
}
