//! Demonstrates loading custom assets using the Amethyst engine.
// TODO: Add asset loader directory store for the meshes.

extern crate amethyst;
extern crate cgmath;
extern crate rayon;

use std::sync::Arc;

use amethyst::{Application, Error, State, Trans};
use amethyst::assets::{BoxedErr, Format, Loader, Source};
use amethyst::config::Config;
use amethyst::core::transform::{LocalTransform, Transform, TransformBundle};
use amethyst::ecs::World;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{Camera, DisplayConfig as DisplayConfig, DrawShaded, Event, KeyboardInput,
                         Light, Material, MaterialDefaults, Mesh, MeshData, Pipeline, PointLight,
                         PosNormTex, Projection, RenderBundle, RenderSystem, Rgba, Stage,
                         VirtualKeyCode, WindowEvent};

struct Custom;

impl Format<Mesh> for Custom {
    const NAME: &'static str = "CUSTOM";

    type Options = ();

    /// Reads the given bytes and produces asset data.
    fn import(&self, name: String, source: Arc<Source>, _: ()) -> Result<MeshData, BoxedErr> {
        let bytes = source.load(&name)?;
        let data: String = String::from_utf8(bytes).unwrap();

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

impl State for AssetsExample {
    fn on_start(&mut self, engine: &mut Engine) {
        engine.world.add_resource(0usize);

        initialise_camera(&mut engine.world);
        initialise_lights(&mut engine.world);

        // Add custom cube object to scene
        let (mesh, mtl) = {
            let mat_defaults = engine.world.read_resource::<MaterialDefaults>();
            let loader = engine.world.read_resource::<Loader>();

            let meshes = &engine.world.read_resource();
            let textures = &engine.world.read_resource();

            let mesh = loader.load("mesh/cuboid.custom", Custom, (), (), meshes);
            let albedo = loader.load_from_data([0.0, 0.0, 1.0, 0.0].into(), textures);
            let mat = Material {
                albedo,
                ..mat_defaults.0.clone()
            };

            (mesh, mat)
        };

        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 0.0, 0.0];
        trans.scale = [2.0, 2.0, 2.0];
        engine
            .world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
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
        "{}/examples/05_assets/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );

    let mut game = Application::build(resources_directory, AssetsExample)
        .expect("Failed to build ApplicationBuilder for an unknown reason.")
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new())?
        .with_local(RenderSystem::build(pipeline_builder, Some(display_config))?)
        .build()?;

    game.run();
    Ok(())
}

fn initialise_camera(world: &mut World) {
    use cgmath::Deg;
    world.add_resource(Camera {
        eye: [0.0, -20.0, 10.0].into(),
        proj: Projection::perspective(1.0, Deg(60.0)).into(),
        forward: [0.0, 20.0, -5.0].into(),
        right: [1.0, 0.0, 0.0].into(),
        up: [0.0, 0.0, 1.0].into(),
    });
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
