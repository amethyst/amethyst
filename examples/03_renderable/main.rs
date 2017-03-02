//! Demonstrates how to load renderable objects, along with several lighting
//! methods.

extern crate amethyst;
extern crate cgmath;

use amethyst::{Application, ElementState, Engine, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::{AssetLoader, DirectoryStore};
use amethyst::config::Element;
use amethyst::ecs::{Join, System, RunArg};
use amethyst::ecs::components::{LocalTransform, Mesh, Renderable, Texture, Transform};
use amethyst::ecs::resources::{Camera, Projection, ScreenDimensions, Time};
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{AmbientLight, DirectionalLight, Layer, PointLight, Pipeline};
use amethyst::renderer::pass::{BlitLayer, Clear, DrawFlat, DrawShaded, Lighting};
use cgmath::{Deg, Euler, Quaternion};
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

impl System<()> for ExampleSystem {
    fn run(&mut self, arg: RunArg, _: ()) {
        let (mut lights, time, mut camera, mut state) = arg.fetch(|w| {
            (w.write::<PointLight>(),
             w.read_resource::<Time>(),
             w.write_resource::<Camera>(),
             w.write_resource::<DemoState>())
        });

        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        state.light_angle -= delta_time;
        state.camera_angle += delta_time / 10.0;

        camera.eye[0] = 20.0 * state.camera_angle.cos();
        camera.eye[1] = 20.0 * state.camera_angle.sin();

        for light in (&mut lights).iter() {
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
        let geom_layer = Layer::new("main",
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
    fn on_start(&mut self, engine: &mut Engine) {
        use amethyst::asset_manager::formats::{Png, Dds, Obj};

        // Set up an assets path by directly registering an assets store.
        let assets_path = format!("{}/examples/03_renderable/assets",
                                  env!("CARGO_MANIFEST_DIR"));
        let store = DirectoryStore::new(assets_path);

        // Create some basic colors and load textures
        let red = Texture::from_color([0.8, 0.2, 0.2, 1.0]);
        let green = Texture::from_color([0.2, 0.8, 0.2, 1.0]);
        let blue = Texture::from_color([0.2, 0.2, 0.8, 1.0]);
        let black = Texture::from_color([0.0, 0.0, 0.0, 1.0]);
        let white = Texture::from_color([1.0, 1.0, 1.0, 1.0]);

        let asset_loader = AssetLoader::new();

        let logo = asset_loader.load(&store, "logo", Png);
        let ground = asset_loader.load(&store, "ground", Dds);

        // Load/generate meshes
        let teapot = asset_loader.load(&store, "teapot", Obj);
        let lid = asset_loader.load(&store, "lid", Obj);
        let rectangle = asset_loader.load(&store, "rectangle", Obj);
        let cube = asset_loader.load(&store, "cube", Obj);
        let cone = asset_loader.load(&store, "cone", Obj);

        let world = engine.planner.mut_world();

        {
            // Let's do other things while the assets are loading
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

        // Add teapot and lid to scene
        for mesh in vec![lid.finish(&mut engine.context), teapot.finish(&mut engine.context)] {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 5.0, 0.0];

            let rend = Renderable::new(mesh.expect("Failed to load mesh"),
                                       red.clone(),
                                       blue.clone(),
                                       white.clone(),
                                       10.0);
            world.create_now()
                .with(rend)
                .with(trans)
                .with(Transform::default())
                .build();
        }

        let cube: Mesh = cube.finish(&mut engine.context).expect("Failed to load cube");
        let logo: Texture = logo.finish(&mut engine.context).expect("Failed to load logo");

        // Add cube to scene
        let rend = Renderable::new(cube.clone(),
                                   logo.clone(),
                                   logo.clone(),
                                   white.clone(),
                                   10.0);
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, -5.0, 2.0];
        trans.scale = [2.0; 3];
        world.create_now()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        let cone = cone.finish(&mut engine.context).expect("Failed to load cone");

        // Add cone to scene
        let rend = Renderable::new(cone, white.clone(), red.clone(), blue.clone(), 40.0);
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 5.0, 0.0];
        trans.scale = [2.0; 3];
        world.create_now()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add custom cube object to scene
        let rend = Renderable::new(cube, blue.clone(), green.clone(), white.clone(), 1.0);
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, -5.0, 1.0];
        world.create_now()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        let rectangle: Mesh = rectangle.finish(&mut engine.context)
            .expect("Failed to load rectangle");
        let ground: Texture = ground.finish(&mut engine.context).expect("Failed to load ground");

        // Create base rectangle as floor
        let rend = Renderable::new(rectangle,
                                   ground.clone(),
                                   ground.clone(),
                                   black.clone(),
                                   1.0);
        let mut trans = LocalTransform::default();
        trans.scale = [10.0; 3];
        world.create_now()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add lights to scene
        world.create_now()
            .with(PointLight::default())
            .build();

        world.create_now()
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
        set_pipeline_state(&mut engine.pipe, true);
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

    fn handle_events(&mut self, events: &[WindowEvent], engine: &mut Engine) -> Trans {

        let w = engine.planner.mut_world();
        let pipe = &mut engine.pipe;

        // Exit if user hits Escape or closes the window
        for e in events {
            match **e {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                    let mut state = w.write_resource::<DemoState>();

                    if state.pipeline_forward {
                        state.pipeline_forward = false;
                        set_pipeline_state(pipe, false);
                    } else {
                        state.pipeline_forward = true;
                        set_pipeline_state(pipe, true);
                    }
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::R)) => {
                    let mut state = w.write_resource::<DemoState>();
                    state.light_color = [0.8, 0.2, 0.2, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::G)) => {
                    let mut state = w.write_resource::<DemoState>();
                    state.light_color = [0.2, 0.8, 0.2, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::B)) => {
                    let mut state = w.write_resource::<DemoState>();
                    state.light_color = [0.2, 0.2, 0.8, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                    let mut state = w.write_resource::<DemoState>();
                    state.light_color = [1.0, 1.0, 1.0, 1.0];
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                    let mut light = w.write_resource::<AmbientLight>();
                    let mut state = w.write_resource::<DemoState>();

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
                    let mut state = w.write_resource::<DemoState>();

                    if state.directional_light {
                        state.directional_light = false;
                        for mut light in (&mut lights).iter() {
                            light.color = [0.0; 4];
                        }
                    } else {
                        state.directional_light = true;
                        for mut light in (&mut lights).iter() {
                            light.color = [0.2; 4];
                        }
                    }
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::P)) => {
                    let mut state = w.write_resource::<DemoState>();

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
    let path = format!("{}/examples/03_renderable/assets/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, cfg)
        .with::<ExampleSystem>(ExampleSystem, "example_system", 1)
        .done();
    game.run();
}
