//! Demonstrates how to load renderable objects, along with several lighting
//! methods.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate cgmath;

use amethyst::{Application, ElementState, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::{AssetManager, DirectoryStore};
use amethyst::config::Config;
use amethyst::ecs::{Fetch, FetchMut, Join, System, WriteStorage, World};
use amethyst::ecs::components::{LocalTransform, Mesh, Texture, Transform};
use amethyst::ecs::resources::{Camera, Projection, ScreenDimensions, Time};
use amethyst::ecs::systems::TransformSystem;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{AmbientLight, DirectionalLight, Layer, PointLight, Pipeline};
use amethyst::renderer::pass::{BlitLayer, Clear, DrawFlat, DrawShaded, Lighting};

use cgmath::{Deg, Euler, Quaternion};
use std::env::set_var;
use std::str;

struct DemoState {
    light_angle: f32,
    light_color: [f32; 4],
    ambient_light: bool,
    point_light: bool,
    directional_light: bool,
    camera_angle: f32,
    pipeline_forward: bool,
}

struct ExampleSystem;

impl<'a> System<'a> for ExampleSystem {
    type SystemData = (WriteStorage<'a, PointLight>,
     Fetch<'a, Time>,
     FetchMut<'a, Camera>,
     FetchMut<'a, DemoState>);

    fn run(&mut self, (mut lights, time, mut camera, mut state): Self::SystemData) {
        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        state.light_angle -= delta_time;
        state.camera_angle += delta_time / 10.0;

        camera.eye[0] = 20.0 * state.camera_angle.cos();
        camera.eye[1] = 20.0 * state.camera_angle.sin();

        for light in (&mut lights).join() {
            light.center[0] = 15.0 * state.light_angle.cos();
            light.center[1] = 15.0 * state.light_angle.sin();
            light.center[2] = 6.0;

            light.color = state.light_color;
        }
    }
}

fn set_pipeline_state(pipe: &mut Pipeline, forward: bool) {
    if forward {
        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                                    DrawShaded::new("main", "main")]);
        pipe.layers = vec![layer];
    } else {
        let geom_layer = Layer::new("gbuffer",
                                    vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                                         DrawFlat::new("main", "main")]);
        let postproc_layer = Layer::new("main",
                                        vec![BlitLayer::new("gbuffer", "ka"),
                                             Lighting::new("main", "gbuffer", "main")]);
        pipe.layers = vec![geom_layer, postproc_layer];
    }
}
struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World, assets: &mut AssetManager, pipe: &mut Pipeline) {
        {
            let dim = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            let proj = Projection::Perspective {
                fov: 60.0,
                aspect_ratio: dim.aspect_ratio,
                near: 1.0,
                far: 100.0,
            };
            camera.proj = proj;
            camera.eye = [0.0, -20.0, 10.0];
            camera.target = [0.0, 0.0, 5.0];
            camera.up = [0.0, 0.0, 1.0];
        }

        // Set up an assets path by directly registering an assets store.
        let assets_path = format!("{}/examples/03_renderable/resources/meshes",
                                  env!("CARGO_MANIFEST_DIR"));
        assets.register_store(DirectoryStore::new(assets_path));

        // Create some basic colors and load textures
        assets.load_asset_from_data::<Texture, [f32; 4]>("red", [0.8, 0.2, 0.2, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("green", [0.2, 0.8, 0.2, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("blue", [0.2, 0.2, 0.8, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("pink", [1.0, 0.8, 0.8, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("black", [0.0, 0.0, 0.0, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        assets.load_asset::<Texture>("logo", "png");
        assets.load_asset::<Texture>("ground", "dds");

        // Load/generate meshes
        assets.load_asset::<Mesh>("teapot", "obj");
        assets.load_asset::<Mesh>("lid", "obj");
        assets.load_asset::<Mesh>("rectangle", "obj");
        assets.load_asset::<Mesh>("cube", "obj");
        assets.load_asset::<Mesh>("cone", "obj");

        // Add teapot and lid to scene
        for mesh in vec!["lid", "teapot"].iter() {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 5.0, 0.0];
            let rend = assets
                .create_renderable(mesh, "red", "blue", "white", 10.0)
                .unwrap();
            world
                .create_entity()
                .with(rend)
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add cube to scene
        let rend = assets
            .create_renderable("cube", "logo", "logo", "white", 1.0)
            .unwrap();
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, -5.0, 2.0];
        trans.scale = [2.0; 3];
        world
            .create_entity()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cone to scene
        let rend = assets
            .create_renderable("cone", "white", "red", "blue", 40.0)
            .unwrap();
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 5.0, 0.0];
        trans.scale = [2.0; 3];
        world
            .create_entity()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add custom cube object to scene
        let rend = assets
            .create_renderable("cube", "blue", "green", "white", 1.0)
            .unwrap();
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, -5.0, 1.0];
        world
            .create_entity()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Create base rectangle as floor
        let rend = assets
            .create_renderable("rectangle", "ground", "ground", "black", 1.0)
            .unwrap();
        let mut trans = LocalTransform::default();
        trans.scale = [10.0; 3];
        world
            .create_entity()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add lights to scene
        world.create_entity().with(PointLight::default()).build();

        world
            .create_entity()
            .with(DirectionalLight {
                      color: [0.2; 4],
                      direction: [-1.0; 3],
                  })
            .build();

        {
            let mut ambient_light = world.write_resource::<AmbientLight>();
            ambient_light.power = 0.01;
        }

        // Set rendering pipeline to forward by default, and add utility resources
        set_pipeline_state(pipe, true);
        world.add_resource::<DemoState>(DemoState {
                                            light_angle: 0.0,
                                            light_color: [1.0; 4],
                                            ambient_light: true,
                                            point_light: true,
                                            directional_light: true,
                                            camera_angle: 0.0,
                                            pipeline_forward: true,
                                        });
    }

    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     w: &mut World,
                     _: &mut AssetManager,
                     pipe: &mut Pipeline)
                     -> Trans {
        // Exit if user hits Escape or closes the window
        let mut state = w.write_resource::<DemoState>();

        for e in events {
            match **e {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                    if state.pipeline_forward {
                        state.pipeline_forward = false;
                        set_pipeline_state(pipe, false);
                    } else {
                        state.pipeline_forward = true;
                        set_pipeline_state(pipe, true);
                    }
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::R)) => {
                    state.light_color = [0.8, 0.2, 0.2, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::G)) => {
                    state.light_color = [0.2, 0.8, 0.2, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::B)) => {
                    state.light_color = [0.2, 0.2, 0.8, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                    state.light_color = [1.0, 1.0, 1.0, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                    let mut light = w.write_resource::<AmbientLight>();

                    if state.ambient_light {
                        state.ambient_light = false;
                        light.power = 0.0;
                    } else {
                        state.ambient_light = true;
                        light.power = 0.01;
                    }
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                    let mut lights = w.write::<DirectionalLight>();

                    if state.directional_light {
                        state.directional_light = false;
                        for mut light in (&mut lights).join() {
                            light.color = [0.0; 4];
                        }
                    } else {
                        state.directional_light = true;
                        for mut light in (&mut lights).join() {
                            light.color = [0.2; 4];
                        }
                    }
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::P)) => {
                    if state.point_light {
                        state.point_light = false;
                        state.light_color = [0.0; 4];
                    } else {
                        state.point_light = true;
                        state.light_color = [1.0; 4];
                    }
                }
                Event::Closed => return Trans::Quit,
                _ => (),
            }
        }
        Trans::None
    }
}

fn main() {
    // Set up an assets path by setting an environment variable. Note that
    // this would normally be done with something like this:
    //
    //     AMETHYST_ASSET_DIRS=/foo/bar cargo run
    let assets_path = format!("{}/examples/03_renderable/resources/textures",
                              env!("CARGO_MANIFEST_DIR"));
    set_var("AMETHYST_ASSET_DIRS", assets_path);

    let path = format!("{}/examples/03_renderable/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::load(path);
    let mut game = Application::build(Example, cfg)
        .with::<ExampleSystem>(ExampleSystem, "example_system", &[])
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
        .done();
    game.run();
}
