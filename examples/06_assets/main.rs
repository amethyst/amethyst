//! Demonstrates several assets-related techniques, including
//! writing a custom asset loader, and loading assets from
//! various paths.

extern crate amethyst;

use std::env::set_var;

use amethyst::asset_manager::{AssetManager, DirectoryStore};
use amethyst::components::rendering::{Mesh, Texture};
use amethyst::components::transform::Transform;
use amethyst::config::Element;
use amethyst::ecs::World;
use amethyst::engine::{Application, State, Trans};
use amethyst::event::{Event, VirtualKeyCode, WindowEvent};
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{Layer, Light};
use amethyst::renderer::{Pipeline};
use amethyst::renderer::pass::{Clear, DrawShaded};
use amethyst::world_resources::camera::{Camera, Projection};
use amethyst::world_resources::ScreenDimensions;

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
            camera.eye = [0.0, 15.0, 0.0];
            camera.target = [0.0, 0.0, 0.0];
            camera.up = [0.0, 0.0, 1.0];
        }

        // Set up an assets path by directly registering an assets store.
        let assets_path = format!("{}/examples/06_assets/resources/meshes",
                       env!("CARGO_MANIFEST_DIR"));
        asset_manager.register_store(DirectoryStore::new(assets_path));

        // Create some basic colors for the teapot, and load some textures
        // for the cube and sphere.
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("dark_blue", [0.0, 0.0, 0.1, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("green", [0.0, 0.0, 0.1, 1.0]);
        asset_manager.load_asset::<Texture>("crate", "png");
        asset_manager.load_asset::<Texture>("grass", "bmp");

        // Load/generate meshes
        asset_manager.load_asset::<Mesh>("Mesh000", "obj");
        asset_manager.load_asset::<Mesh>("Mesh001", "obj");
        // asset_manager.gen_cube("cube");
        // asset_manager.gen_sphere("sphere", 32, 32);

        // Add teapot lid to scene
        let renderable = asset_manager.create_renderable("Mesh000", "dark_blue", "green").unwrap();
        world.create_now()
            .with(renderable)
            .with(Transform::default())
            .build();

        // Add teapot body to scene
        let renderable = asset_manager.create_renderable("Mesh001", "dark_blue", "green").unwrap();
        world.create_now()
            .with(renderable)
            .with(Transform::default())
            .build();

        // // Add cube to scene
        // let translation = cgmath::Vector3::new(0.0, 2.5, -5.0);
        // let transform: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(translation).into();
        // let fragment = asset_manager.get_fragment("cube", "crate", "crate", transform).unwrap();
        // ctx.renderer.add_fragment("main", fragment);
        //
        // // Add sphere to scene
        // let translation = cgmath::Vector3::new(0.0, -2.5, -5.0);
        // let transform: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(translation).into();
        // let fragment = asset_manager.get_fragment("sphere", "grass", "green", transform).unwrap();
        // ctx.renderer.add_fragment("main", fragment);

        // Add light to scene
        let light = Light {
            color: [1.0, 1.0, 1.0, 1.0],
            radius: 10.0,
            center: [15.0, 0.0, 0.0],
            propagation_constant: 0.2,
            propagation_linear: 0.2,
            propagation_r_square: 0.6,
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

    fn handle_events(&mut self, events: &[WindowEvent], _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
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
