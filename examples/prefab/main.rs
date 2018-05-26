//! Demonstrates loading prefabs using the Amethyst engine.

extern crate amethyst;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, Result as AssetResult, RonFormat,
                       SimpleFormat};
use amethyst::config::Config;
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::ecs::prelude::World;
use amethyst::input::InputBundle;
use amethyst::renderer::{Camera, DisplayConfig, DrawShaded, Event, GraphicsPrefab, KeyboardInput,
                         Light, Mesh, MeshData, Pipeline, PointLight, PosNormTex, Projection,
                         RenderBundle, Rgba, Stage, TextureFormat, VirtualKeyCode, WindowEvent};
use amethyst::{Application, Error, GameData, GameDataBuilder, State, StateData, Trans};

#[derive(Clone, Deserialize, Serialize)]
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

type MyPrefabData = (GraphicsPrefab<Custom, TextureFormat>, Transform);

struct AssetsExample;

impl<'a, 'b> State<GameData<'a, 'b>> for AssetsExample {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        world.add_resource(0usize);

        initialise_camera(world);
        initialise_lights(world);

        let prefab_handle = world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab.ron", RonFormat, (), ())
        });
        world.create_entity().with(prefab_handle).build();
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        // If the user pressed the escape key, or requested the window to be closed,
                        // quit the application.
                        Trans::Quit
                    }
                    _ => Trans::None,
                }
            }
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
        ::std::process::exit(1);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    // Add our meshes directory to the asset loader.
    let resources_directory = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/examples/asset_loading/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipeline_builder, Some(display_config)))?;

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
        center: [5.0, -20.0, 15.0].into(),
        intensity: 100.0,
        radius: 1.0,
        color: Rgba::white(),
        ..Default::default()
    }.into();

    world.create_entity().with(light).build();
}
