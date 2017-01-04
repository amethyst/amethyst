//! Demonstrates several assets-related techniques, including
//! writing a custom asset loader, and loading assets from
//! various paths. Also demonstrates basic lighting te

extern crate amethyst;
extern crate cgmath;

use std::env::set_var;
use std::str;

use amethyst::asset_manager::{Assets, AssetLoader, AssetLoaderRaw, AssetManager, DirectoryStore};
use amethyst::components::rendering::{Mesh, Texture};
use amethyst::components::transform::{LocalTransform, Transform};
use amethyst::config::Element;
use amethyst::ecs::{Join, Processor, RunArg, World};
use amethyst::engine::{Application, State, Trans};
use amethyst::event::WindowEvent;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{Layer, PointLight};
use amethyst::renderer::{Pipeline, VertexPosNormal};
use amethyst::renderer::pass::{BlitLayer, Clear, DrawFlat, DrawShaded, Lighting};
use amethyst::world_resources::camera::{Camera, Projection};
use amethyst::world_resources::{ScreenDimensions, Time};
use cgmath::{Deg, Euler, Quaternion};


// Implement custom asset loader that reads files with a simple format of
// 1 vertex and 1 normal per line, with coordinates separated by whitespace.
struct CustomObj {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
}

impl AssetLoaderRaw for CustomObj {
    fn from_raw(_: &Assets, data: &[u8]) -> Option<CustomObj> {
        let data: String = str::from_utf8(data).unwrap().into();
        let mut vertices = vec![];
        let mut normals = vec![];

        for line in data.split("\n") {
            if line.len() < 1 {
                continue
            }

            let nums: Vec<&str> = line.split_whitespace().collect();

            vertices.push([
                nums[0].parse::<f32>().unwrap(),
                nums[1].parse::<f32>().unwrap(),
                nums[2].parse::<f32>().unwrap(),
            ]);

            normals.push([
                nums[3].parse::<f32>().unwrap(),
                nums[4].parse::<f32>().unwrap(),
                nums[5].parse::<f32>().unwrap(),
            ]);
        }

        Some(CustomObj {
            vertices: vertices,
            normals: normals,
        })
    }
}

impl AssetLoader<Mesh> for CustomObj {
    fn from_data(assets: &mut Assets, obj: CustomObj) -> Option<Mesh> {
        let vertices = obj.vertices.iter().zip(obj.normals.iter()).map(|(v, n)| {
            VertexPosNormal {
                pos: v.clone(),
                normal: n.clone(),
                tex_coord: [0.0, 0.0],
            }
        }).collect::<Vec<_>>();
        AssetLoader::<Mesh>::from_data(assets, vertices)
    }
}


struct Angles {
    light: f32,
    camera: f32,
}

struct PipelineState {
    forward: bool,
}

struct ExampleProcessor;


impl Processor<()> for ExampleProcessor {
    fn run(&mut self, arg: RunArg, _: ()) {
        let (
            mut lights,
            time,
            mut angles,
            mut camera,
        ) = arg.fetch(|w| (
            w.write::<PointLight>(),
            w.read_resource::<Time>(),
            w.write_resource::<Angles>(),
            w.write_resource::<Camera>(),
        ));

        let delta_time = time.delta_time.subsec_nanos() as f32 / 1.0e9;

        angles.light += delta_time;
        angles.camera -= delta_time / 10.0;

        camera.eye[0] = 20.0 * angles.camera.cos();
        camera.eye[1] = 20.0 * angles.camera.sin();

        for light in (&mut lights).iter() {
            light.center[0] = 15.0 * angles.light.cos();
            light.center[1] = 15.0 * angles.light.sin();
            light.center[2] = 6.0;
        }
    }
}


fn set_pipeline_state(pipeline: &mut Pipeline, forward: bool) {
    if forward {
        pipeline.layers = vec![
            Layer::new("main", vec![
                Clear::new([0.0, 0.0, 0.0, 1.0]),
                DrawShaded::new("main", "main"),
            ]),
        ];
    } else {
        pipeline.layers = vec![
            Layer::new("gbuffer",
                vec![
                    Clear::new([0., 0., 0., 1.]),
                    DrawFlat::new("main", "main"),
                ]
            ),
            Layer::new("main",
                vec![
                    BlitLayer::new("gbuffer", "ka"),
                    BlitLayer::new("gbuffer", "kd"),
                    BlitLayer::new("gbuffer", "ks"),
                    BlitLayer::new("gbuffer", "normal"),
                    Lighting::new("main", "gbuffer", "main"),
                ]
            ),
        ];
    }
}


struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World, asset_manager: &mut AssetManager, pipeline: &mut Pipeline) {

        {
            let dimensions = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            let proj = Projection::Perspective {
                fov: 60.0,
                aspect_ratio: dimensions.aspect_ratio,
                near: 1.0,
                far: 100.0,
            };
            camera.projection = proj;
            camera.eye = [0.0, -20.0, 10.0];
            camera.target = [0.0, 0.0, 5.0];
            camera.up = [0.0, 0.0, 1.0];
        }

        // Set up an assets path by directly registering an assets store.
        let assets_path = format!("{}/examples/06_assets/resources/meshes",
                       env!("CARGO_MANIFEST_DIR"));
        asset_manager.register_store(DirectoryStore::new(assets_path));

        // Create some basic colors and load textures
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("green", [0.0, 1.0, 0.2, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("tan",   [0.8, 0.6, 0.5, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("blue",  [0.0, 0.0, 0.6, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("red",   [0.6, 0.0, 0.0, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("pink",  [1.0, 0.8, 0.8, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("green", [0.0, 0.1, 0.0, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("black", [0.0, 0.0, 0.0, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        asset_manager.load_asset::<Texture>("logo", "png");
        asset_manager.load_asset::<Texture>("grass", "png");

        // Load/generate meshes
        asset_manager.load_asset::<Mesh>("teapot", "obj");
        asset_manager.load_asset::<Mesh>("lid", "obj");
        asset_manager.load_asset::<Mesh>("rectangle", "obj");
        asset_manager.load_asset::<Mesh>("cube", "obj");
        asset_manager.load_asset::<Mesh>("cone", "obj");

        // Also add custom asset loader and load mesh
        asset_manager.register_loader::<Mesh, CustomObj>("custom");
        asset_manager.load_asset::<Mesh>("cuboid", "custom");

        // Add teapot and lid to scene
        for mesh in vec!["lid", "teapot"].iter() {
            let mut transform = LocalTransform::default();
            transform.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            transform.translation = [5.0, 5.0, 0.0];
            let renderable = asset_manager.create_renderable(mesh, "blue", "green", "white", 1.0).unwrap();
            world.create_now()
                .with(renderable)
                .with(transform)
                .with(Transform::default())
                .build();
        }

        // Add cube to scene
        let renderable = asset_manager.create_renderable("cube", "logo", "logo", "white", 1.0).unwrap();
        let mut transform = LocalTransform::default();
        transform.translation = [5.0, -5.0, 2.0];
        transform.scale = [2.0, 2.0, 2.0];
        world.create_now()
            .with(renderable)
            .with(transform)
            .with(Transform::default())
            .build();

        // Add cone to scene
        let renderable = asset_manager.create_renderable("cone", "red", "pink", "white", 1.0).unwrap();
        let mut transform = LocalTransform::default();
        transform.translation = [-5.0, 5.0, 0.0];
        transform.scale = [2.0, 2.0, 2.0];
        world.create_now()
            .with(renderable)
            .with(transform)
            .with(Transform::default())
            .build();

        // Add custom cube object to scene
        let renderable = asset_manager.create_renderable("cuboid", "blue", "green", "white", 1.0).unwrap();
        let mut transform = LocalTransform::default();
        transform.translation = [-5.0, -5.0, 1.0];
        world.create_now()
            .with(renderable)
            .with(transform)
            .with(Transform::default())
            .build();

        // Create base rectangle as floor
        let renderable = asset_manager.create_renderable("rectangle", "grass", "grass", "black", 1.0).expect("1234");
        let mut transform = LocalTransform::default();
        transform.scale = [10.0, 10.0, 10.0];
        world.create_now()
            .with(renderable)
            .with(transform)
            .with(Transform::default())
            .build();

        // Add light to scene
        world.create_now()
            .with(PointLight::from_radius(5.0))
            .build();

        // Set rendering pipeline to forward by default, and add utility resources
        set_pipeline_state(pipeline, true);
        world.add_resource::<Angles>(Angles { light: 0.0, camera: 0.0 });
        world.add_resource::<PipelineState>(PipelineState { forward: true });
    }

    fn handle_events(&mut self, events: &[WindowEvent], w: &mut World, _: &mut AssetManager, pipeline: &mut Pipeline) -> Trans {
        // Exit if user hits Escape or closes the window
        use amethyst::event::*;
        for event in events {
            match event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Space)) => {
                    let mut pipeline_state = w.write_resource::<PipelineState>();

                    if pipeline_state.forward {
                        pipeline_state.forward = false;
                        set_pipeline_state(pipeline, false);
                    } else {
                        pipeline_state.forward = true;
                        set_pipeline_state(pipeline, true);
                    }
                },
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
    let assets_path = format!("{}/examples/06_assets/resources/textures",
                   env!("CARGO_MANIFEST_DIR"));
    set_var("AMETHYST_ASSET_DIRS", assets_path);

    let path = format!("{}/examples/06_assets/resources/config.yml",
                   env!("CARGO_MANIFEST_DIR"));
    let display_config = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, display_config)
        .with::<ExampleProcessor>(ExampleProcessor, "example_processor", 1)
        .done();
    game.run();
}
