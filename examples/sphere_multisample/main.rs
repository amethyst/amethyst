//! Displays a shaded sphere with multisampling enabled to the user.

extern crate amethyst;

use amethyst::assets::Loader;
use amethyst::core::cgmath::Deg;
use amethyst::core::transform::GlobalTransform;
use amethyst::ecs::prelude::World;
use amethyst::input::{is_close_requested, is_key};
use amethyst::prelude::*;
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, Event, Light, Mesh,
                         Pipeline, PointLight, PosNormTex, Projection, RenderBundle, Rgba, Shape,
                         Stage, VirtualKeyCode};

const SPHERE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0]; // blue
const AMBIENT_LIGHT_COLOUR: Rgba = Rgba(0.01, 0.01, 0.01, 1.0); // near-black
const POINT_LIGHT_COLOUR: Rgba = Rgba(1.0, 1.0, 1.0, 1.0); // white
const BACKGROUND_COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 0.0]; // black
const LIGHT_POSITION: [f32; 3] = [2.0, 2.0, -2.0];
const LIGHT_RADIUS: f32 = 5.0;
const LIGHT_INTENSITY: f32 = 3.0;

struct Example;

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        // Initialise the scene with an object, a light and a camera.
        initialise_sphere(world);
        initialise_lights(world);
        initialise_camera(world);
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
    let display_config_path = format!(
        "{}/examples/sphere_multisample/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target(BACKGROUND_COLOUR, 1.0)
            .with_pass(DrawShaded::<PosNormTex>::new()),
    );

    let config = DisplayConfig::load(&display_config_path);

    let game_data = GameDataBuilder::default().with_bundle(RenderBundle::new(pipe, Some(config)))?;
    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}

/// This function initialises a sphere and adds it to the world.
fn initialise_sphere(world: &mut World) {
    // Create a sphere mesh and material.

    use amethyst::assets::Handle;
    use amethyst::renderer::{Material, MaterialDefaults};

    let (mesh, material) = {
        let loader = world.read_resource::<Loader>();

        let mesh: Handle<Mesh> = loader.load_from_data(
            Shape::Sphere(32, 32).generate::<Vec<PosNormTex>>(None),
            (),
            &world.read_resource(),
        );

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
        .with(GlobalTransform::default())
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
    use amethyst::core::cgmath::Matrix4;
    let transform =
        Matrix4::from_translation([0.0, 0.0, -4.0].into()) * Matrix4::from_angle_y(Deg(180.));
    world
        .create_entity()
        .with(Camera::from(Projection::perspective(1.3, Deg(60.0))))
        .with(GlobalTransform(transform.into()))
        .build();
}
