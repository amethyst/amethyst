//! Displays a shaded sphere to the user.

extern crate amethyst;
extern crate cgmath;
extern crate futures;
extern crate genmesh;

use amethyst::assets::Loader;
use amethyst::ecs::World;
use amethyst::ecs::rendering::{AmbientColor, RenderBundle};
use amethyst::ecs::transform::Transform;
use amethyst::prelude::*;
use amethyst::renderer::{Config as DisplayConfig, Mesh, Rgba, Texture};
use amethyst::renderer::prelude::*;
use cgmath::{Deg, Vector3};
use cgmath::prelude::InnerSpace;
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

const SPHERE_COLOUR: Rgba = Rgba(0.0, 0.0, 1.0, 1.0); // blue
const AMBIENT_LIGHT_COLOUR: Rgba = Rgba(0.01, 0.01, 0.01, 1.0); // near-black
const POINT_LIGHT_COLOUR: Rgba = Rgba(1.0, 1.0, 1.0, 1.0); // white
const BACKGROUND_COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 0.0]; // black
const LIGHT_POSITION: [f32; 3] = [2.0, 2.0, -2.0];
const LIGHT_RADIUS: f32 = 5.0;
const LIGHT_INTENSITY: f32 = 3.0;

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        // Initialise the scene with an object, a light and a camera.
        initialise_sphere(&mut engine.world);
        initialise_lights(&mut engine.world);
        initialise_camera(&mut engine.world);
    }

    fn handle_event(&mut self, _: &mut Engine, event: Event) -> Trans {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } |
                WindowEvent::Closed => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}


type DrawShaded = pass::DrawShaded<PosNormTex, AmbientColor, Mesh, Material, Transform, Light>;

fn run() -> Result<(), amethyst::Error> {
    let display_config_path = format!(
        "{}/examples/02_sphere/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let mut game = Application::build(Example)?
        .with_bundle(
            RenderBundle::new(
                Pipeline::build().with_stage(
                    Stage::with_backbuffer()
                        .clear_target(BACKGROUND_COLOUR, 1.0)
                        .with_pass(DrawShaded::new()),
                ),
            ).with_config(DisplayConfig::load(&display_config_path)),
        )?
        .build()?;
    Ok(game.run())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosNormTex> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            PosNormTex {
                position: [x, y, z],
                normal: Vector3::from([x, y, z]).normalize().into(),
                tex_coord: [0.1, 0.1],
            }
        })
        .triangulate()
        .vertices()
        .collect()
}

/// This function initialises a sphere and adds it to the world.
fn initialise_sphere(world: &mut World) {
    // Create a sphere mesh and material.

    let (mesh, material) = {
        let loader = world.read_resource::<Loader>();

        let mesh = loader.load_from_data(gen_sphere(32, 32), &world.read_resource());
        //let material = MaterialBuilder::new().with_albedo(Texture::from_color_val(SPHERE_COLOUR));
        //
        //// Load the material.
        //let material = load_proc_asset(world, move |world| {
        //    let factory = world.read_resource::<Factory>();
        //    factory
        //        .create_material(material)
        //        .map(MaterialComponent)
        //        .map_err(BoxedErr::new)
        //});

        (mesh, ())
    };


    // Create a sphere entity using the mesh and the material.
    world
        .create_entity()
        .with(Transform::default())
        .with(mesh)
        //.with(material)
        .build();
}

/// This function adds an ambient light and a point light to the world.
fn initialise_lights(world: &mut World) {
    // Add ambient light.
    world.add_resource(AmbientColor(AMBIENT_LIGHT_COLOUR));

    // Add point light.
    world
        .create_entity()
        .with(
            PointLight {
                center: LIGHT_POSITION.into(),
                radius: LIGHT_RADIUS,
                intensity: LIGHT_INTENSITY,
                color: POINT_LIGHT_COLOUR,
                ..Default::default()
            }.into(),
        )
        .build();
}


/// This function initialises a camera and adds it to the world.
fn initialise_camera(world: &mut World) {
    world.add_resource(Camera {
        eye: [0.0, 0.0, -4.0].into(),
        proj: Projection::perspective(1.3, Deg(60.0)).into(),
        forward: [0.0, 0.0, 1.0].into(),
        right: [1.0, 0.0, 0.0].into(),
        up: [0.0, 1.0, 0.0].into(),
    });
}
