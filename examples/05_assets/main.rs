//! Demonstrates several assets-related techniques, including writing a custom
//! asset loader, and loading assets from various paths.

extern crate amethyst;
extern crate cgmath;

use amethyst::{Application, Engine, Event, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::asset_manager::{AssetFormat, AssetLoader, DirectoryStore, Import, ImportError};
use amethyst::config::Element;
use amethyst::ecs::components::{LocalTransform, Renderable, Texture, Transform};
use amethyst::ecs::resources::{Camera, Projection, ScreenDimensions};
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{Layer, PointLight, VertexPosNormal};
use amethyst::renderer::pass::{Clear, DrawShaded};
use cgmath::{Deg, Euler, Quaternion};
use std::str;

// Implement custom asset loader that reads files with a simple format of
// 1 vertex and 1 normal per line, with coordinates separated by whitespace.
struct CustomObj;

impl AssetFormat for CustomObj {
    fn file_extensions(&self) -> &[&str] {
        const FE: [&'static str; 1] = ["custom"];
        const FE_REF: &'static [&'static str; 1] = &FE;

        FE_REF
    }
}

impl Import<Vec<VertexPosNormal>> for CustomObj {
    fn import(&self, data: Box<[u8]>) -> Result<Vec<VertexPosNormal>, ImportError> {
        let data: String = str::from_utf8(data.as_ref())?.to_string();
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();

        let trimmed: Vec<&str> = data.lines()
            .filter(|line| line.len() >= 1)
            .collect();

        for line in trimmed {
            let nums: Result<_, _> = line.split_whitespace().map(|x| x.parse::<f32>()).collect();
            let nums: Vec<f32> =
                nums.map_err(|x| ImportError::FormatError(format!("Invalid float: {:?}", x)))?;

            vertices.push([nums[0], nums[1], nums[2]]);
            normals.push([nums[3], nums[4], nums[5]]);
        }

        let vertices = vertices.iter()
            .zip(&normals)
            .map(|(v, n)| {
                VertexPosNormal {
                    pos: v.clone(),
                    normal: n.clone(),
                    tex_coord: [0.0, 0.0],
                }
            })
            .collect::<Vec<_>>();

        Ok(vertices)
    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        use amethyst::asset_manager::formats::{Png, Bmp, Obj};

        let world = engine.planner.mut_world();

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
        let assets_path = format!("{}/examples/05_assets/assets", env!("CARGO_MANIFEST_DIR"));
        let store = DirectoryStore::new(assets_path);

        // Create some basic colors for the teapot, and load some textures
        // for the cube and sphere.
        let dark_blue = Texture::from_color([0.0, 0.0, 0.1, 1.0]);
        let green = Texture::from_color([0.0, 1.0, 0.2, 1.0]);
        let tan = Texture::from_color([0.8, 0.6, 0.5, 1.0]);
        let white = Texture::from_color([1.0, 1.0, 1.0, 1.0]);

        let asset_loader = AssetLoader::new();

        let mycrate = asset_loader.load(&store, "crate", Png);
        let grass = asset_loader.load(&store, "grass", Bmp);

        // Load/generate meshes
        let teapot = asset_loader.load(&store, "teapot", Obj);
        let lid = asset_loader.load(&store, "lid", Obj);
        let cube = asset_loader.load(&store, "cube", Obj);
        let sphere = asset_loader.load(&store, "sphere", Obj);
        let cuboid = asset_loader.load(&store, "cuboid", CustomObj);

        // Add teapot and lid to scene
        for mesh in vec![lid, teapot] {
            let mut trans = LocalTransform::default();
            trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(-90.0), Deg(0.0))).into();
            trans.translation = [5.0, 0.0, 5.0];
            let mesh = mesh.finish(&mut engine.context).expect("Failed to load mesh");
            let rend = Renderable::new(mesh, dark_blue.clone(), green.clone(), white.clone(), 1.0);
            world.create_now()
                .with(rend)
                .with(trans)
                .with(Transform::default())
                .build();
        }

        // Add custom cube object to scene
        let cuboid = cuboid.finish(&mut engine.context).expect("Failed to load cuboid");
        let rend = Renderable::new(cuboid, dark_blue, green.clone(), white.clone(), 1.0);
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 0.0, 0.0];
        trans.scale = [2.0, 2.0, 2.0];
        world.create_now()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add cube to scene
        let cube = cube.finish(&mut engine.context).expect("Failed to load cube");
        let mycrate = mycrate.finish(&mut engine.context).expect("Failed to load crate");
        let rend = Renderable::new(cube, mycrate, tan, white.clone(), 1.0);
        let mut trans = LocalTransform::default();
        trans.translation = [5.0, 0.0, 0.0];
        trans.scale = [2.0, 2.0, 2.0];
        world.create_now()
            .with(rend)
            .with(trans)
            .with(Transform::default())
            .build();

        // Add sphere to scene
        let sphere = sphere.finish(&mut engine.context).expect("Failed to load sphere");
        let grass = grass.finish(&mut engine.context).expect("Failed to load grass");

        let rend = Renderable::new(sphere, grass, green, white, 1.0);
        let mut trans = LocalTransform::default();
        trans.translation = [-5.0, 0.0, 7.5];
        trans.rotation = Quaternion::from(Euler::new(Deg(90.0), Deg(0.0), Deg(0.0))).into();
        trans.scale = [0.15, 0.15, 0.15];
        world.create_now()
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

        world.create_now()
            .with(light)
            .build();

        // Set up rendering pipeline
        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                                    DrawShaded::new("main", "main")]);
        engine.pipe.layers.push(layer);
    }

    fn handle_events(&mut self, events: &[WindowEvent], _: &mut Engine) -> Trans {
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
    let path = format!("{}/examples/05_assets/assets/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, cfg).done();
    game.run();
}
