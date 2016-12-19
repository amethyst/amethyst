extern crate amethyst;
extern crate cgmath;
extern crate obj;

use amethyst::engine::{Application, State, Trans};
use amethyst::gfx_device::DisplayConfig;
use amethyst::config::Element;
use amethyst::ecs::World;
use amethyst::asset_manager::{Assets, AssetManager, AssetLoader, AssetLoaderRaw, DirectoryStore};
use amethyst::components::rendering::{Mesh, Texture};
use amethyst::event::WindowEvent;
use amethyst::renderer::{VertexPosNormal, Pipeline};
use cgmath::{InnerSpace, Vector3};
use std::io::BufReader;

struct Obj(obj::Obj);

impl AssetLoaderRaw for Obj {
    fn from_raw(_: &Assets, data: &[u8]) -> Option<Obj> {
        obj::load_obj(BufReader::new(data)).ok().map(|obj| Obj(obj))
    }
}

impl AssetLoader<Mesh> for Obj {
    fn from_data(assets: &mut Assets, obj: Obj) -> Option<Mesh> {
        let obj = obj.0;
        let vertices = obj.indices.iter().map(|&index| {
            let vertex = obj.vertices[index as usize];
            let normal = vertex.normal;
            let normal = Vector3::from(normal).normalize();
            VertexPosNormal {
                pos: vertex.position,
                normal: normal.into(),
                tex_coord: [0., 0.],
            }
        }).collect::<Vec<VertexPosNormal>>();

        AssetLoader::<Mesh>::from_data(assets, vertices)
    }
}

struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World, asset_manager: &mut AssetManager, pipeline: &mut Pipeline) {
        use amethyst::renderer::pass::{Clear, DrawShaded};
        use amethyst::renderer::{Layer, Light};
        use amethyst::components::transform::Transform;
        use amethyst::world_resources::camera::{Camera, Projection};
        use amethyst::world_resources::ScreenDimensions;

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
            camera.eye = [10.0, 10.0, 0.0];
            camera.target = [0.0, 3.0, 0.0];
            camera.up = [0.0, 1.0, 0.0];
        }

        asset_manager.register_asset::<Mesh>();
        asset_manager.register_asset::<Texture>();

        asset_manager.register_loader::<Mesh, Obj>("obj");

        let assets_path = format!("{}/examples/06_assets/resources/assets",
                       env!("CARGO_MANIFEST_DIR"));
        asset_manager.register_store(DirectoryStore::new(assets_path));

        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("dark_blue", [0.0, 0.0, 0.1, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("green", [0.0, 0.0, 0.1, 1.0]);
        asset_manager.load_asset::<Mesh>("Mesh000", "obj");
        asset_manager.load_asset::<Mesh>("Mesh001", "obj");

        let renderable = asset_manager.create_renderable("Mesh000", "dark_blue", "green").unwrap();
        world.create_now()
            .with(renderable)
            .with(Transform::default())
            .build();

        let renderable = asset_manager.create_renderable("Mesh001", "dark_blue", "green").unwrap();
        world.create_now()
            .with(renderable)
            .with(Transform::default())
            .build();

        let light = Light {
            color: [1.0, 1.0, 1.0, 1.0],
            radius: 10.0,
            center: [6.0, 6.0, 6.0],
            propagation_constant: 0.2,
            propagation_linear: 0.2,
            propagation_r_square: 0.6,
        };

        world.create_now()
            .with(light)
            .build();

        let layer =
            Layer::new("main",
                        vec![
                            Clear::new([0.0, 0.0, 0.0, 1.0]),
                            DrawShaded::new("main", "main"),
                        ]);
        pipeline.layers = vec![layer];
    }

    fn handle_events(&mut self, events: &[WindowEvent], _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        // Exit if user hits Escape or closes the window
        use amethyst::event::*;
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
    let path = format!("{}/examples/06_assets/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let display_config = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, display_config).done();
    game.run();
}
