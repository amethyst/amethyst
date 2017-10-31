//! Displays a shaded sphere to the user.

extern crate amethyst;
extern crate genmesh;

use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::Vector3;
use amethyst::core::cgmath::prelude::InnerSpace;
use amethyst::core::transform::Transform;
use amethyst::ecs::World;
use amethyst::prelude::*;
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, Light, Mesh, Pipeline,
                         PngFormat, PointLight, PosNormTex, RenderBundle, RenderSystem, Rgba,
                         ScreenDimensions, Stage, Texture};
use amethyst::ui::{DrawUi, FontAsset, TtfFormat, UiBundle, UiImage, UiText, UiTransform};
use amethyst::winit::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

const SPHERE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0]; // blue
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
        let (logo, font) = {
            let loader = engine.world.read_resource::<Loader>();

            let logo = loader.load(
                "texture/logo_transparent.png",
                PngFormat,
                Default::default(),
                (),
                &engine.world.read_resource::<AssetStorage<Texture>>(),
            );

            let font = loader.load(
                "font/square.ttf",
                TtfFormat,
                Default::default(),
                (),
                &engine.world.read_resource::<AssetStorage<FontAsset>>(),
            );
            (logo, font)
        };

        engine
            .world
            .create_entity()
            .with(UiTransform::new(
                "logo".to_string(),
                300.,
                300.,
                0.,
                232.,
                266.,
            ))
            .with(UiImage {
                texture: logo.clone(),
            })
            .build();
        engine
            .world
            .create_entity()
            .with(UiTransform::new(
                "hello_world".to_string(),
                0.,
                0.,
                1.,
                500.,
                500.,
            ))
            .with(UiText::new(
                font,
                "Hello world!".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                75.,
            ))
            .build();
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
                } => Trans::Quit,
                _ => Trans::None,
            },
            _ => Trans::None,
        }
    }
}

fn run() -> Result<(), amethyst::Error> {
    let display_config_path = format!(
        "{}/examples/09_ui/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&display_config_path);

    let mut game = Application::build(resources, Example)?
        .with_bundle(RenderBundle::new())?
        .with_bundle(UiBundle::new(&[]))?;
    let pipe = {
        let loader = game.world.read_resource();
        let mesh_storage = game.world.read_resource();

        Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target(BACKGROUND_COLOUR, 1.0)
                .with_pass(DrawShaded::<PosNormTex>::new())
                .with_pass(DrawUi::new(&loader, &mesh_storage)),
        )
    };
    game = game.with_local(RenderSystem::build(pipe, Some(config))?);
    Ok(game.build()?.run())
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

    use amethyst::assets::Handle;
    use amethyst::renderer::{Material, MaterialDefaults};

    let (mesh, material) = {
        let loader = world.read_resource::<Loader>();

        let mesh: Handle<Mesh> =
            loader.load_from_data(gen_sphere(32, 32).into(), (), &world.read_resource());

        let albedo = SPHERE_COLOUR.into();

        let tex_storage = world.read_resource();
        let mat_defaults = world.read_resource::<MaterialDefaults>();

        let albedo = loader.load_from_data(albedo, (), &tex_storage);

        let mat = Material {
            albedo,
            ..mat_defaults.0.clone()
        };

        (mesh, mat)
    };


    // Create a sphere entity using the mesh and the material.
    world
        .create_entity()
        .with(Transform::default())
        .with(mesh)
        .with(material)
        .build();
}

/// This function adds an ambient light and a point light to the world.
fn initialise_lights(world: &mut World) {
    // Add ambient light.
    world.add_resource(AmbientColor(AMBIENT_LIGHT_COLOUR));

    let light: Light = PointLight {
        center: LIGHT_POSITION.into(),
        radius: LIGHT_RADIUS,
        intensity: LIGHT_INTENSITY,
        color: POINT_LIGHT_COLOUR,
        ..Default::default()
    }.into();

    // Add point light.
    world.create_entity().with(light).build();
}


/// This function initialises a camera and adds it to the world.
fn initialise_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    world.add_resource(Camera::standard_3d(width, height));
}
