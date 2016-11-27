//! Opens an empty window.

extern crate amethyst;
extern crate genmesh;
extern crate cgmath;

use amethyst::engine::{Application, State, Trans};
use amethyst::config::Element;
use amethyst::ecs::World;
use amethyst::gfx_device::DisplayConfig;
use amethyst::asset_manager::AssetManager;
use amethyst::context::event::EngineEvent;
use amethyst::gfx_device::{Renderable, Texture, Mesh};
use amethyst::renderer::VertexPosNormal;
use amethyst::processors::transform::LocalTransform;

use self::genmesh::generators::{SphereUV};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, InnerSpace};

struct Example;

impl State for Example {
    fn on_start(&mut self, world: &mut World, asset_manager: &mut AssetManager) {
        // use amethyst::renderer::pass::Clear;
        // use amethyst::renderer::Layer;
        // let clear_layer =
        //     Layer::new("main",
        //                 vec![
        //                     Clear::new([0.0, 0.0, 0.0, 1.0]),
        //                 ]);
        // let pipeline = vec![clear_layer];
        // gfx_device.set_pipeline(pipeline);
        // let black = AssetLoader::<Texture>::from_data(asset_manager, [0.0, 0.0, 0.0, 1.0]);
        let sphere_vertices = gen_sphere(32, 32);
        let sphere_mesh = asset_manager.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(sphere_vertices).unwrap();
        let dark_blue = asset_manager.load_asset_from_data::<Texture, [f32; 4]>([0.0, 0.0, 0.1, 1.0]).unwrap();
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
    }

    fn handle_events(&mut self, events: &[EngineEvent], _: &mut World, _: &mut AssetManager) -> Trans {
        use amethyst::context::event::*;
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

// impl AssetManager {
//     /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
//     /// and the number of vertices from pole to pole (v).
//     pub fn gen_sphere(&mut self, name: &str, u: usize, v: usize) {
//         let data: Vec<VertexPosNormal> = SphereUV::new(u, v)
//             .vertex(|(x, y, z)| {
//                 VertexPosNormal {
//                     pos: [x, y, z],
//                     normal: Vector3::new(x, y, z).normalize().into(),
//                     tex_coord: [0., 0.],
//                 }
//             })
//             .triangulate()
//             .vertices()
//             .collect();
//         self.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(name, data);
//     }
//     /// Generate and load a cube mesh.
//     pub fn gen_cube(&mut self, name: &str) {
//         let data: Vec<VertexPosNormal> = Cube::new()
//             .vertex(|(x, y, z)| {
//                 VertexPosNormal {
//                     pos: [x, y, z],
//                     normal: Vector3::new(x, y, z).normalize().into(),
//                     tex_coord: [0., 0.],
//                 }
//             })
//             .triangulate()
//             .vertices()
//             .collect();
//         self.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(name, data);
//     }
//     /// Generate and load a rectangle mesh in XY plane with given `width` and `height`.
//     pub fn gen_rectangle(&mut self, name: &str, width: f32, height: f32) {
//         let data = vec![
//             VertexPosNormal {
//                 pos: [-width/2., height/2., 0.],
//                 normal: [0., 0., 1.],
//                 tex_coord: [0., 1.],
//             },
//             VertexPosNormal {
//                 pos: [-width/2., -height/2., 0.],
//                 normal: [0., 0., 1.],
//                 tex_coord: [0., 0.],
//             },
//             VertexPosNormal {
//                 pos: [width/2., -height/2., 0.],
//                 normal: [0., 0., 1.],
//                 tex_coord: [1., 0.],
//             },
//             VertexPosNormal {
//                 pos: [width/2., -height/2., 0.],
//                 normal: [0., 0., 1.],
//                 tex_coord: [0., 1.],
//             },
//             VertexPosNormal {
//                 pos: [width/2., height/2., 0.],
//                 normal: [0., 0., 1.],
//                 tex_coord: [0., 0.],
//             },
//             VertexPosNormal {
//                 pos: [-width/2., height/2., 0.],
//                 normal: [0., 0., 1.],
//                 tex_coord: [1., 0.],
//             },
//         ];
//         self.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>(name, data);
//     }

//     /// Create a constant solid color `Texture` from a specified color.
//     pub fn create_constant_texture(&mut self, name: &str, color: [f32; 4]) {
//         self.load_asset_from_data::<Texture, [f32; 4]>(name, color);
//     }
// }
