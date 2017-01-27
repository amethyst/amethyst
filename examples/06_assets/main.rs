//! Demonstrates several assets-related techniques, including
//! writing a custom asset loader, and loading assets from
//! various paths.

extern crate amethyst;
extern crate cgmath;

use std::env::set_var;
use std::str;

use amethyst::asset_manager::{AssetLoader, AssetLoaderRaw, AssetManager, Assets, DirectoryStore};
use amethyst::components::rendering::{Mesh, Texture};
use amethyst::components::transform::{LocalTransform, Transform};
use amethyst::config::Element;
use amethyst::specs::World;
use amethyst::engine::{Application, State, Trans};
use amethyst::event::{Event, VirtualKeyCode, WindowEvent};
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{Layer, PointLight, Pipeline, VertexPosNormal};
use amethyst::renderer::pass::{Clear, DrawShaded};
use amethyst::world_resources::camera::{Camera, Projection};
use amethyst::world_resources::ScreenDimensions;
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
                continue;
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
        let vertices = obj.vertices
            .iter()
            .zip(obj.normals.iter())
            .map(|(v, n)| {
                VertexPosNormal {
                    pos: v.clone(),
                    normal: n.clone(),
                    tex_coord: [0.0, 0.0],
                }
            })
            .collect::<Vec<_>>();
        AssetLoader::<Mesh>::from_data(assets, vertices)
    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self,
                world: &mut World,
                asset_manager: &mut AssetManager,
                pipeline: &mut Pipeline) {
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
        let assets_path = format!("{}/examples/06_assets/resources/meshes", env!("CARGO_MANIFEST_DIR"));
        asset_manager.register_store(DirectoryStore::new(assets_path));

        // Create some basic colors for the teapot, and load some textures
        // for the cube and sphere.
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("dark_blue", [0.0, 0.0, 0.1, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("green", [0.0, 1.0, 0.2, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("tan", [0.8, 0.6, 0.5, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        asset_manager.load_asset::<Texture>("crate", "png");
        asset_manager.load_asset::<Texture>("grass", "bmp");

        // Load/generate meshes
        asset_manager.load_asset::<Mesh>("teapot", "obj");
        asset_manager.load_asset::<Mesh>("lid", "obj");
        asset_manager.load_asset::<Mesh>("cube", "obj");
        asset_manager.load_asset::<Mesh>("sphere", "obj");

        // Also add custom asset loader and load mesh
        asset_manager.register_loader::<Mesh, CustomObj>("custom");
        asset_manager.load_asset::<Mesh>("cuboid", "custom");

        // Add teapot and lid to scene
        for mesh in vec!["lid", "teapot"].iter() {
            let mut transform = LocalTransform::default();
            transform.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            transform.translation = [5.0, 0.0, 5.0];
            let renderable = asset_manager.create_renderable(mesh, "dark_blue", "green", "white", 1.0).unwrap();
            world.create_now()
                .with(renderable)
                .with(transform)
                .with(Transform::default())
                .build();
        }

        // Add custom cube object to scene
        let renderable = asset_manager.create_renderable("cuboid", "dark_blue", "green", "white", 1.0).unwrap();
        let mut transform = LocalTransform::default();
        transform.translation = [-5.0, 0.0, 0.0];
        transform.scale = [2.0, 2.0, 2.0];
        world.create_now()
            .with(renderable)
            .with(transform)
            .with(Transform::default())
            .build();

        // Add cube to scene
        let renderable = asset_manager.create_renderable("cube", "crate", "tan", "white", 1.0).unwrap();
        let mut transform = LocalTransform::default();
        transform.translation = [5.0, 0.0, 0.0];
        transform.scale = [2.0, 2.0, 2.0];
        world.create_now()
            .with(renderable)
            .with(transform)
            .with(Transform::default())
            .build();

        // Add sphere to scene
        let renderable = asset_manager.create_renderable("sphere", "grass", "green", "white", 1.0).unwrap();
        let mut transform = LocalTransform::default();
        transform.translation = [-5.0, 0.0, 7.5];
        transform.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(0.0), Deg(0.0))).into();
        transform.scale = [0.15, 0.15, 0.15];
        world.create_now()
            .with(renderable)
            .with(transform)
            .with(Transform::default())
            .build();

        // Add light to scene
        let light = PointLight {
            center: [5.0, -20.0, 15.0],
            intensity: 10.0,
            radius: 100.0,
            ..Default::default()
        };

        world.create_now()
            .with(light)
            .build();

        // Set up rendering pipeline
        let layer = Layer::new("main", vec![
            Clear::new([0.0, 0.0, 0.0, 1.0]),
            DrawShaded::new("main", "main"),
        ]);
        pipeline.layers = vec![layer];
    }

    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     _: &mut World,
                     _: &mut AssetManager,
                     _: &mut Pipeline)
                     -> Trans {
        // Exit if user hits Escape or closes the window
        for event in events {
            match event.payload {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => return Trans::Quit,
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
    let mut game = Application::build(Example, display_config).done();
    game.run();
}
