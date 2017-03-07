//! Demonstrates several assets-related techniques, including writing a custom
//! asset loader, and loading assets from various paths.
//!
//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate cgmath;

use amethyst::{Application, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::{AssetLoader, AssetLoaderRaw, AssetManager, Assets, DirectoryStore};
use amethyst::config::Config;
use amethyst::ecs::World;
use amethyst::ecs::components::{LocalTransform, Mesh, Texture, Transform};
use amethyst::ecs::resources::{Camera, Projection, ScreenDimensions};
use amethyst::ecs::systems::TransformSystem;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{Layer, PointLight, Pipeline, VertexPosNormal};
use amethyst::renderer::pass::{Clear, DrawShaded};
use cgmath::{Deg, Euler, Quaternion};
use std::env::set_var;
use std::str;

// Implement custom asset loader that reads files with a simple format of
// 1 vertex and 1 normal per line, with coordinates separated by whitespace.
struct CustomObj {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
}

impl AssetLoaderRaw for CustomObj {
    fn from_raw(_: &Assets, data: &[u8]) -> Option<CustomObj> {
        let data: String = str::from_utf8(data).unwrap().into();
        let mut vertices = Vec::new();
        let mut normals = Vec::new();

        let trimmed: Vec<&str> = data.lines().filter(|line| line.len() >= 1).collect();

        for line in trimmed {
            let nums: Vec<&str> = line.split_whitespace().collect();

            vertices.push([nums[0].parse::<f32>().unwrap(),
                           nums[1].parse::<f32>().unwrap(),
                           nums[2].parse::<f32>().unwrap()]);

            normals.push([nums[3].parse::<f32>().unwrap(),
                          nums[4].parse::<f32>().unwrap(),
                          nums[5].parse::<f32>().unwrap()]);
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
        let assets_path = format!("{}/examples/05_assets/resources/meshes",
                                  env!("CARGO_MANIFEST_DIR"));
        assets.register_store(DirectoryStore::new(assets_path));

        // Create some basic colors for the teapot, and load some textures
        // for the cube and sphere.
        assets.load_asset_from_data::<Texture, [f32; 4]>("dark_blue", [0.0, 0.0, 0.1, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("green", [0.0, 1.0, 0.2, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("tan", [0.8, 0.6, 0.5, 1.0]);
        assets.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        assets.load_asset::<Texture>("crate", "png");
        assets.load_asset::<Texture>("grass", "bmp");

        // Load/generate meshes
        assets.load_asset::<Mesh>("teapot", "obj");
        assets.load_asset::<Mesh>("lid", "obj");
        assets.load_asset::<Mesh>("cube", "obj");
        assets.load_asset::<Mesh>("sphere", "obj");

        // Also add custom asset loader and load mesh
        assets.register_loader::<Mesh, CustomObj>("custom");
        assets.load_asset::<Mesh>("cuboid", "custom");

        // Add teapot and lid to scene
        for mesh in vec!["lid", "teapot"].iter() {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 0.0, 5.0];
            let rend = assets
                .create_renderable(mesh, "dark_blue", "green", "white", 1.0)
                .unwrap();
            world
                .create_entity()
                .with(rend)
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add custom cube object to scene
        let rend = assets
            .create_renderable("cuboid", "dark_blue", "green", "white", 1.0)
            .unwrap();
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 0.0, 0.0];
        trans.scale = [2.0, 2.0, 2.0];
        world
            .create_entity()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cube to scene
        let rend = assets
            .create_renderable("cube", "crate", "tan", "white", 1.0)
            .unwrap();
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, 0.0, 0.0];
        trans.scale = [2.0, 2.0, 2.0];
        world
            .create_entity()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add sphere to scene
        let rend = assets
            .create_renderable("sphere", "grass", "green", "white", 1.0)
            .unwrap();
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 0.0, 7.5];
        trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(0.0), Deg(0.0))).into();
        trans.scale = [0.15, 0.15, 0.15];
        world
            .create_entity()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add light to scene
        let light = PointLight {
            center: [5.0, -20.0, 15.0],
            intensity: 10.0,
            radius: 100.0,
            ..Default::default()
        };

        world.create_entity().with(light).build();

        // Set up rendering pipeline
        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                                    DrawShaded::new("main", "main")]);
        pipe.layers.push(layer);
    }

    fn handle_events(&mut self,
                     events: &[WindowEvent],
                     _: &mut World,
                     _: &mut AssetManager,
                     _: &mut Pipeline)
                     -> Trans {
        // Exit if user hits Escape or closes the window
        for e in events {
            match **e {
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
    let assets_path = format!("{}/examples/05_assets/resources/textures",
                              env!("CARGO_MANIFEST_DIR"));
    set_var("AMETHYST_ASSET_DIRS", assets_path);

    let path = format!("{}/examples/05_assets/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::load(path);
    let mut game = Application::build(Example, cfg)
        .with::<TransformSystem>(TransformSystem::new(), "transform_system", &[])
        .done();
    game.run();
}
