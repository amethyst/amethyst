//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate cgmath;

use amethyst::{Application, Error, State, Trans};
use amethyst::assets::{Loader, Progress};
use amethyst::config::Config;
use amethyst::ecs::{Fetch, FetchMut, Join, System, World, WriteStorage};
use amethyst::ecs::rendering::{create_render_system, AmbientColor, RenderBundle};
use amethyst::ecs::transform::{LocalTransform, Transform, TransformBundle};
use amethyst::prelude::*;
use amethyst::renderer::{Camera, Config as DisplayConfig, MaterialDefaults, MeshHandle, Rgba};
use amethyst::renderer::formats::{ObjFormat, PngFormat};
use amethyst::renderer::prelude::*;
use amethyst::timing::Time;
use cgmath::{Deg, Euler, Quaternion};

struct DemoState {
    light_angle: f32,
    light_color: [f32; 4],
    ambient_light: bool,
    point_light: bool,
    directional_light: bool,
    camera_angle: f32,
    #[allow(dead_code)]
    pipeline_forward: bool, // TODO
}

struct ExampleSystem;

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (
        WriteStorage<'a, Light>,
        Fetch<'a, Time>,
        FetchMut<'a, Camera>,
        FetchMut<'a, DemoState>,
    );

    fn run(&mut self, (mut lights, time, mut camera, mut state): Self::SystemData) {
        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        let light_angular_velocity = -1.0;
        let light_orbit_radius = 15.0;
        let light_z = 6.0;

        let camera_angular_velocity = 0.1;
        let camera_orbit_radius = 20.0;

        state.light_angle += light_angular_velocity * delta_time;
        state.camera_angle += camera_angular_velocity * delta_time;

        let target = camera.eye + camera.forward;
        camera.eye[0] = camera_orbit_radius * state.camera_angle.cos();
        camera.eye[1] = camera_orbit_radius * state.camera_angle.sin();
        camera.forward = target - camera.eye;

        for point_light in (&mut lights).join().filter_map(
            |light| if let Light::Point(ref mut point_light) = *light {
                Some(point_light)
            } else {
                None
            },
        ) {
            point_light.center[0] = light_orbit_radius * state.light_angle.cos();
            point_light.center[1] = light_orbit_radius * state.light_angle.sin();
            point_light.center[2] = light_z;

            point_light.color = state.light_color.into();
        }
    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        initialise_camera(&mut engine.world.write_resource::<Camera>());

        let assets = load_assets(&engine.world);

        // Add teapot and lid to scene
        for mesh in vec![assets.lid.clone(), assets.teapot.clone()] {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 5.0, 0.0];

            engine
                .world
                .create_entity()
                .with(mesh)
                .with(assets.red.clone())
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add cube to scene
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, -5.0, 2.0];
        trans.scale = [2.0; 3];

        engine
            .world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.logo.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cone to scene
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 5.0, 0.0];
        trans.scale = [2.0; 3];

        engine
            .world
            .create_entity()
            .with(assets.cone.clone())
            .with(assets.white.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        // Add custom cube object to scene
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, -5.0, 1.0];
        engine
            .world
            .create_entity()
            .with(assets.cube.clone())
            .with(assets.red.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        // Create base rectangle as floor
        let mut trans = LocalTransform::default();
        trans.scale = [10.0; 3];

        engine
            .world
            .create_entity()
            .with(assets.rectangle.clone())
            .with(assets.white.clone())
            .with(trans)
            .with(Transform::default())
            .build();

        let light: Light = PointLight {
            color: [1.0, 1.0, 0.0].into(),
            intensity: 50.0,
            ..PointLight::default()
        }.into();

        // Add lights to scene
        engine.world.create_entity().with(light).build();

        let light: Light = DirectionalLight {
            color: [0.2; 4].into(),
            direction: [-1.0; 3].into(),
        }.into();

        engine.world.create_entity().with(light).build();

        {
            engine
                .world
                .add_resource(AmbientColor(Rgba::from([0.01; 3])));
        }

        engine.world.add_resource::<DemoState>(DemoState {
            light_angle: 0.0,
            light_color: [1.0; 4],
            ambient_light: true,
            point_light: true,
            directional_light: true,
            camera_angle: 0.0,
            pipeline_forward: true,
        });
    }

    fn handle_event(&mut self, engine: &mut Engine, event: Event) -> Trans {
        let w = &mut engine.world;
        // Exit if user hits Escape or closes the window
        let mut state = w.write_resource::<DemoState>();

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Closed => return Trans::Quit,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        match virtual_keycode {
                            Some(VirtualKeyCode::Escape) => return Trans::Quit,
                            Some(VirtualKeyCode::Space) => {
                                // TODO: figure out how to change pipeline
                            /*if state.pipeline_forward {
                                state.pipeline_forward = false;
                                set_pipeline_state(pipe, false);
                            } else {
                                state.pipeline_forward = true;
                                set_pipeline_state(pipe, true);
                            }*/
                            }
                            Some(VirtualKeyCode::R) => {
                                state.light_color = [0.8, 0.2, 0.2, 1.0];
                            }
                            Some(VirtualKeyCode::G) => {
                                state.light_color = [0.2, 0.8, 0.2, 1.0];
                            }
                            Some(VirtualKeyCode::B) => {
                                state.light_color = [0.2, 0.2, 0.8, 1.0];
                            }
                            Some(VirtualKeyCode::W) => {
                                state.light_color = [1.0, 1.0, 1.0, 1.0];
                            }
                            Some(VirtualKeyCode::A) => {
                                let mut color = w.write_resource::<AmbientColor>();
                                if state.ambient_light {
                                    state.ambient_light = false;
                                    color.0 = [0.0; 3].into();
                                } else {
                                    state.ambient_light = true;
                                    color.0 = [0.01; 3].into();
                                }
                            }
                            Some(VirtualKeyCode::D) => {
                                let mut lights = w.write::<Light>();

                                if state.directional_light {
                                    state.directional_light = false;
                                    for light in (&mut lights).join() {
                                        if let Light::Directional(ref mut d) = *light {
                                            d.color = [0.0; 4].into();
                                        }
                                    }
                                } else {
                                    state.directional_light = true;
                                    for light in (&mut lights).join() {
                                        if let Light::Directional(ref mut d) = *light {
                                            d.color = [0.2; 4].into();
                                        }
                                    }
                                }
                            }
                            Some(VirtualKeyCode::P) => if state.point_light {
                                state.point_light = false;
                                state.light_color = [0.0; 4].into();
                            } else {
                                state.point_light = true;
                                state.light_color = [1.0; 4].into();
                            },
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        Trans::None
    }
}

struct Assets {
    cube: MeshHandle,
    cone: MeshHandle,
    lid: MeshHandle,
    rectangle: MeshHandle,
    teapot: MeshHandle,

    red: Material,
    white: Material,
    logo: Material,
}

fn load_assets(world: &World) -> Assets {
    let mesh_storage = world.read_resource();
    let tex_storage = world.read_resource();
    let mat_defaults = world.read_resource::<MaterialDefaults>();
    let loader = world.read_resource::<Loader>();
    let mut progress = Progress::new();

    let red = loader.load_from_data([1.0, 0.0, 0.0, 1.0].into(), &tex_storage);
    let red = Material {
        albedo: red,
        ..mat_defaults.0.clone()
    };

    let white = loader.load_from_data([1.0, 1.0, 1.0, 1.0].into(), &tex_storage);
    let white = Material {
        albedo: white,
        ..mat_defaults.0.clone()
    };

    let logo = Material {
        albedo: loader.load(
            "logo.png",
            PngFormat,
            Default::default(),
            &mut progress,
            &tex_storage,
        ),
        ..mat_defaults.0.clone()
    };

    let cube = loader.load("cube.obj", ObjFormat, (), &mut progress, &mesh_storage);
    let cone = loader.load("cone.obj", ObjFormat, (), &mut progress, &mesh_storage);
    let lid = loader.load("lid.obj", ObjFormat, (), &mut progress, &mesh_storage);
    let teapot = loader.load("teapot.obj", ObjFormat, (), &mut progress, &mesh_storage);
    let rectangle = loader.load("rectangle.obj", ObjFormat, (), &mut progress, &mesh_storage);

    Assets {
        cube,
        cone,
        lid,
        rectangle,
        teapot,

        red,
        white,
        logo,
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
    let resources_directory = format!(
        "{}/examples/03_renderable/resources",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config_path = format!(
        "{}/examples/03_renderable/resources/config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let display_config = DisplayConfig::load(display_config_path);
    let pipeline_builder = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawShaded::new()),
    );

    let mut game = Application::build(resources_directory, Example)?
        .with::<ExampleSystem>(ExampleSystem, "example_system", &[])
        .with_bundle(TransformBundle::new().with_dep(&["example_system"]))?
        .with_bundle(RenderBundle::new())?
        .with_local(create_render_system(
            pipeline_builder,
            Some(display_config),
        )?)
        .build()?;

    game.run();
    Ok(())
}

type DrawShaded = pass::DrawShaded<PosNormTex, AmbientColor, Transform>;

/// Initialises the camera structure.
fn initialise_camera(camera: &mut Camera) {
    use cgmath::Deg;

    // TODO: Fix the aspect ratio.
    camera.proj = Projection::perspective(1.0, Deg(60.0)).into();
    camera.eye = [0.0, -20.0, 10.0].into();

    camera.forward = [0.0, 20.0, -5.0].into();
    camera.right = [1.0, 0.0, 0.0].into();
    camera.up = [0.0, 0.0, 1.0].into();
}
