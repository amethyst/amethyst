//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate cgmath;
extern crate futures;

use std::str;

use amethyst::{Application, Error, State, Trans};
use amethyst::assets::{AssetFuture, BoxedErr, Context, Format, Loader};
use amethyst::assets::formats::meshes::ObjFormat;
use amethyst::assets::formats::textures::PngFormat;
use amethyst::config::Config;
use amethyst::ecs::{Fetch, FetchMut, Join, System, WriteStorage};
use amethyst::ecs::rendering::{AmbientColor, Factory, LightComponent, MaterialComponent,
                               MeshComponent, MeshContext, TextureComponent, TextureContext};
use amethyst::ecs::transform::{LocalTransform, Transform};
use amethyst::prelude::*;
use amethyst::renderer::{Camera, Config as DisplayConfig, Rgba};
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
    pipeline_forward: bool, // TODO
}

struct ExampleSystem;

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (
        WriteStorage<'a, LightComponent>,
        Fetch<'a, Time>,
        FetchMut<'a, Camera>,
        FetchMut<'a, DemoState>,
    );

    fn run(&mut self, (mut lights, time, mut camera, mut state): Self::SystemData) {
        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        state.light_angle -= delta_time;
        state.camera_angle += delta_time / 10.0;

        let target = camera.eye + camera.forward;
        camera.eye[0] = 20.0 * state.camera_angle.cos();
        camera.eye[1] = 20.0 * state.camera_angle.sin();
        camera.forward = target - camera.eye;

        for point_light in (&mut lights).join().filter_map(|light| {
            if let LightComponent(Light::Point(ref mut point_light)) = *light {
                Some(point_light)
            } else {
                None
            }
        }) {
            point_light.center[0] = 15.0 * state.light_angle.cos();
            point_light.center[1] = 15.0 * state.light_angle.sin();
            point_light.center[2] = 6.0;

            point_light.color = state.light_color.into();
        }
    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        initialise_camera(&mut engine.world.write_resource::<Camera>());

        // Add teapot and lid to scene
        for mesh in vec!["lid", "teapot"].iter() {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 5.0, 0.0];
            let mesh = load_mesh(engine, mesh, ObjFormat);
            let mtl = make_material(engine, [1.0, 0.0, 0.0, 1.0]);
            engine
                .world
                .create_entity()
                .with(mesh)
                .with(mtl)
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add cube to scene
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, -5.0, 2.0];
        trans.scale = [2.0; 3];
        let mesh = load_mesh(engine, "cube", ObjFormat);
        let mtl = load_material(engine, "logo", PngFormat);
        engine
            .world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cone to scene
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 5.0, 0.0];
        trans.scale = [2.0; 3];
        let mesh = load_mesh(engine, "cone", ObjFormat);
        let mtl = make_material(engine, [1.0; 4]);
        engine
            .world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add custom cube object to scene
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, -5.0, 1.0];
        let mesh = load_mesh(engine, "cube", ObjFormat);
        let mtl = make_material(engine, [0.0, 0.0, 1.0, 1.0]);
        engine
            .world
            .create_entity()
            .with(mesh)
            .with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();

        // Create base rectangle as floor
        let mut trans = LocalTransform::default();
        trans.scale = [10.0; 3];
        let mesh = load_mesh(engine, "rectangle", ObjFormat);
        //let mtl = load_material(engine, "ground", DdsFormat);
        engine.world
            .create_entity()
            .with(mesh)
            //.with(mtl)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add lights to scene
        engine
            .world
            .create_entity()
            .with(LightComponent(
                PointLight {
                    color: [1.0, 1.0, 0.0].into(),
                    intensity: 50.0,
                    ..PointLight::default()
                }.into(),
            ))
            .build();

        engine
            .world
            .create_entity()
            .with(LightComponent(
                DirectionalLight {
                    color: [0.2; 4].into(),
                    direction: [-1.0; 3].into(),
                }.into(),
            ))
            .build();

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
                        input: KeyboardInput {
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
                                let mut lights = w.write::<LightComponent>();

                                if state.directional_light {
                                    state.directional_light = false;
                                    for light in (&mut lights).join() {
                                        if let LightComponent(Light::Directional(ref mut d)) =
                                            *light
                                        {
                                            d.color = [0.0; 4].into();
                                        }
                                    }
                                } else {
                                    state.directional_light = true;
                                    for light in (&mut lights).join() {
                                        if let LightComponent(Light::Directional(ref mut d)) =
                                            *light
                                        {
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



fn main() {
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
        ::std::process::exit(1);
    }
}

/// Wrapper around the main, so we can return errors easily.
fn run() -> Result<(), Error> {
    use amethyst::assets::Directory;
    use amethyst::ecs::transform::{Child, Init, LocalTransform, TransformSystem};
    use amethyst::ecs::common::Errors;

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
            .with_model_pass(pass::DrawShaded::<PosNormTex>::new()),
    );

    let mut game = Application::build(Example)
        .expect("Failed to build ApplicationBuilder for an unknown reason.")
        .register::<Child>()
        .register::<LocalTransform>()
        .register::<Init>()
        .with::<ExampleSystem>(ExampleSystem, "example_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
        .with_renderer(pipeline_builder, Some(display_config))?
        .add_store("resources", Directory::new(resources_directory))
        .add_resource(Errors::new())
        .build()?;

    game.run();
    Ok(())
}


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

fn load_material<F>(engine: &mut Engine, albedo: &str, format: F) -> AssetFuture<MaterialComponent>
where
    F: Format + 'static,
    F::Data: Into<<TextureContext as Context>::Data>,
{
    use futures::Future;
    let future = {
        let factory = engine.world.read_resource::<Factory>();
        factory
            .create_material(MaterialBuilder::new())
            .map_err(BoxedErr::new)
    }.join({
        let loader = engine.world.read_resource::<Loader>();
        loader.load_from::<TextureComponent, _, _, _>(albedo, format, "resources")
    })
        .map(|(mut mtl, albedo)| {
            mtl.albedo = albedo.0.inner();
            MaterialComponent(mtl)
        });
    AssetFuture::from_future(future)
}

fn make_material(engine: &mut Engine, albedo: [f32; 4]) -> AssetFuture<MaterialComponent> {
    use futures::Future;
    let future = {
        let factory = engine.world.read_resource::<Factory>();
        factory
            .create_material(
                MaterialBuilder::new().with_albedo(TextureBuilder::from_color_val(albedo)),
            )
            .map(MaterialComponent)
            .map_err(BoxedErr::new)
    };
    AssetFuture::from_future(future)
}

fn load_mesh<F>(engine: &mut Engine, name: &str, f: F) -> AssetFuture<MeshComponent>
where
    F: Format + 'static,
    F::Data: Into<<MeshContext as Context>::Data>,
{
    let future = {
        let loader = engine.world.read_resource::<Loader>();
        loader.load_from::<MeshComponent, _, _, _>(name, f, "resources")
    };
    future
}
