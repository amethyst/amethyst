//! Displays a shaded sphere to the user.

extern crate amethyst;
extern crate genmesh;
#[macro_use]
extern crate log;

use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::Time;
use amethyst::core::cgmath::{Deg, InnerSpace, Vector3};
use amethyst::core::transform::GlobalTransform;
use amethyst::ecs::{Entity, World};
use amethyst::prelude::*;
use amethyst::input::InputBundle;
use amethyst::renderer::{AmbientColor, Camera, DisplayConfig, DrawShaded, Light, Mesh, Pipeline,
                         PngFormat, PointLight, PosNormTex, Projection, RenderBundle, Rgba, Stage,
                         Texture};
use amethyst::ui::{DrawUi, FontAsset, TextEditing, TtfFormat, UiBundle, UiFocused, UiImage,
                   UiText, UiTransform,MouseReactive,UiEvent};
use amethyst::utils::fps_counter::{FPSCounter, FPSCounterBundle};
use amethyst::winit::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;
use amethyst::shrev::{ReaderId,EventChannel};
use amethyst::ecs::{System,FetchMut};

const SPHERE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0]; // blue
const AMBIENT_LIGHT_COLOUR: Rgba = Rgba(0.01, 0.01, 0.01, 1.0); // near-black
const POINT_LIGHT_COLOUR: Rgba = Rgba(1.0, 1.0, 1.0, 1.0); // white
const BACKGROUND_COLOUR: [f32; 4] = [0.0, 0.0, 0.0, 0.0]; // black
const LIGHT_POSITION: [f32; 3] = [2.0, 2.0, -2.0];
const LIGHT_RADIUS: f32 = 5.0;
const LIGHT_INTENSITY: f32 = 3.0;

struct Example {
    fps_display: Option<Entity>,
}

impl State for Example {
    fn on_start(&mut self, world: &mut World) {
        // Initialise the scene with an object, a light and a camera.
        initialise_sphere(world);
        initialise_lights(world);
        initialise_camera(world);
        let (logo, font) = {
            let loader = world.read_resource::<Loader>();

            let logo = loader.load(
                "texture/logo_transparent.png",
                PngFormat,
                Default::default(),
                (),
                &world.read_resource::<AssetStorage<Texture>>(),
            );

            let font = loader.load(
                "font/square.ttf",
                TtfFormat,
                Default::default(),
                (),
                &world.read_resource::<AssetStorage<FontAsset>>(),
            );
            (logo, font)
        };

        world
            .create_entity()
            .with(UiTransform::new(
                "logo".to_string(),
                300.,
                300.,
                0.,
                232.,
                266.,
                0,
            ))
            .with(UiImage {
                texture: logo.clone(),
            })
            .with(MouseReactive)
            .build();

        let text = world
            .create_entity()
            .with(UiTransform::new(
                "hello_world".to_string(),
                0.,
                200.,
                1.,
                500.,
                75.,
                1,
            ))
            .with(UiText::new(
                font.clone(),
                "Hello world!".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                75.,
            ))
            .with(TextEditing::new(
                12,
                [0.0, 0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
                false,
            ))
            .build();
        let fps = world
            .create_entity()
            .with(UiTransform::new(
                "fps".to_string(),
                0.,
                0.,
                1.,
                500.,
                75.,
                2,
            ))
            .with(UiText::new(
                font,
                "N/A".to_string(),
                [1.0, 1.0, 1.0, 1.0],
                75.,
            ))
            .build();
        self.fps_display = Some(fps);
        world.write_resource::<UiFocused>().entity = Some(text);
    }

    fn update(&mut self, world: &mut World) -> Trans {
        let mut ui_text = world.write::<UiText>();
        if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
            if world.read_resource::<Time>().frame_number() % 20 == 0 {
                let fps = world.read_resource::<FPSCounter>().sampled_fps();
                fps_display.text = format!("FPS: {:.*}", 2, fps);
            }
        }

        Trans::None
    }

    fn handle_event(&mut self, _: &mut World, event: Event) -> Trans {
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
        "{}/examples/ui/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let resources = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));
    let config = DisplayConfig::load(&display_config_path);
    let pipe = {
        Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target(BACKGROUND_COLOUR, 1.0)
                .with_pass(DrawShaded::<PosNormTex>::new())
                .with_pass(DrawUi::new()),
        )
    };
    let mut game = Application::build(resources, Example { fps_display: None })?
        .with_bundle(UiBundle::new())?
        .with(UiEventHandlerSystem::new(),"ui_event_handler",&[])
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?
        .build()?;
    game.run();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Failed to execute example: {}", e);
        ::std::process::exit(1);
    }
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosNormTex> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| PosNormTex {
            position: [x, y, z],
            normal: Vector3::from([x, y, z]).normalize().into(),
            tex_coord: [0.1, 0.1],
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


/// This shows how to handle UI events.
pub struct UiEventHandlerSystem{
    reader_id: Option<ReaderId<UiEvent>>,
}

impl UiEventHandlerSystem{
    pub fn new() -> Self {
        UiEventHandlerSystem{
            reader_id: None,
        }
    }
}

impl<'a> System<'a> for UiEventHandlerSystem{
    type SystemData = FetchMut<'a, EventChannel<UiEvent>>;

    fn run(&mut self, mut events: Self::SystemData) {
        if self.reader_id.is_none(){
            self.reader_id = Some(events.register_reader());
        }
        for ev in events.read(self.reader_id.as_mut().unwrap()){
            info!("You just interacted with a ui element: {:?}", ev);
        }
    }
}