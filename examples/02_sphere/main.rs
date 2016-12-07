//! Displays a multicolored sphere to the user.

extern crate amethyst;
extern crate genmesh;
extern crate cgmath;

use amethyst::engine::{Application, State, Trans};
use amethyst::config::Element;
use amethyst::ecs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::asset_manager::AssetManager;
use amethyst::components::event::EngineEvent;
use amethyst::renderer::{VertexPosNormal, Pipeline};

use self::genmesh::generators::{SphereUV};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, InnerSpace};

struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World, asset_manager: &mut AssetManager, pipeline: &mut Pipeline) {
        use amethyst::renderer::pass::{Clear, DrawShaded};
        use amethyst::renderer::{Layer, Light};
        use amethyst::world_resources::camera::{Projection, Camera};
        use amethyst::world_resources::ScreenDimensions;
        use amethyst::components::transform::LocalTransform;
        use amethyst::components::rendering::{Texture, Mesh, Renderable};
        let clear_layer =
            Layer::new("main",
                        vec![
                            Clear::new([0.0, 0.0, 0.0, 1.0]),
                            DrawShaded::new("main", "main"),
                        ]);
        pipeline.layers = vec![clear_layer];
        {
            let dimensions = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            camera.projection = Projection::Perspective {
                fov: 90.0,
                aspect_ratio: dimensions.aspect_ratio,
                near: 0.1,
                far: 100.0,
            };
            camera.eye = [5.0, 0.0, 0.0];
            camera.target = [0.0, 0.0, 0.0];
        }
        let sphere_vertices = gen_sphere(32, 32);
        let sphere_mesh = asset_manager.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(sphere_vertices).unwrap();
        let dark_blue = asset_manager.load_asset_from_data::<Texture, [f32; 4]>([0.0, 0.0, 0.01, 1.0]).unwrap();
        let green = asset_manager.load_asset_from_data::<Texture, [f32; 4]>([0.0, 1.0, 0.0, 1.0]).unwrap();
        let sphere = Renderable {
            ka: dark_blue,
            kd: green,
            mesh: sphere_mesh,
        };
        let transform = LocalTransform::default();
        world.create_now()
            .with::<Renderable>(sphere)
            .with::<LocalTransform>(transform)
            .build();
        let light = Light {
            color: [1.0, 1.0, 1.0, 1.0],
            radius: 1.0,
            center: [2.0, 2.0, 2.0],
            propagation_constant: 0.0,
            propagation_linear: 0.0,
            propagation_r_square: 1.0,
        };
        world.create_now()
            .with::<Light>(light)
            .build();
    }

    fn handle_events(&mut self, events: &[EngineEvent], _: &mut World, _: &mut AssetManager, _: &mut Pipeline) -> Trans {
        use amethyst::components::event::*;
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
    let path = format!("{}/examples/01_window/resources/config.yml",
                        env!("CARGO_MANIFEST_DIR"));
    let display_config = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, display_config).done();
    game.run();
}


fn gen_sphere(u: usize, v: usize) -> Vec<VertexPosNormal> {
    let data: Vec<VertexPosNormal> = SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            VertexPosNormal {
                pos: [x, y, z],
                normal: Vector3::new(x, y, z).normalize().into(),
                tex_coord: [0., 0.],
            }
        })
        .triangulate()
        .vertices()
        .collect();
    data
}
