//! Displays a multicolored sphere to the user.

extern crate amethyst;
extern crate cgmath;
extern crate genmesh;

use amethyst::{Application, Event, Engine, State, Trans, VirtualKeyCode, WindowEvent};
use amethyst::config::Element;
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::VertexPosNormal;
use cgmath::{Vector3, InnerSpace};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;

struct Example;

impl State for Example {
    fn on_start(&mut self, engine: &mut Engine) {
        use amethyst::asset_manager::Asset;
        use amethyst::ecs::components::{Mesh, Renderable, Texture};
        use amethyst::ecs::resources::{Camera, Projection, ScreenDimensions};
        use amethyst::renderer::{Layer, PointLight};
        use amethyst::renderer::pass::{Clear, DrawShaded};

        let layer = Layer::new("main",
                               vec![Clear::new([0.0, 0.0, 0.0, 1.0]),
                                    DrawShaded::new("main", "main")]);

        engine.pipe.layers.push(layer);

        let world = engine.planner.mut_world();

        {
            let dim = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            camera.proj = Projection::Perspective {
                fov: 60.0,
                aspect_ratio: dim.aspect_ratio,
                near: 0.1,
                far: 100.0,
            };
            camera.eye = [5.0, 0.0, 0.0];
            camera.target = [0.0, 0.0, 0.0];
        }

        let sphere_verts = gen_sphere(32, 32);
        let sphere_mesh = Mesh::from_data(sphere_verts, &mut engine.context)
            .expect("Failed to load sphere mesh");
        let blue = Texture::from_color([0.0, 0.0, 1.0, 1.0]);
        let white = Texture::from_color([1.0, 1.0, 1.0, 1.0]);

        let sphere = Renderable::new(sphere_mesh, blue, white.clone(), white, 1.0);

        world.create_now()
            .with(sphere)
            .build();

        let light = PointLight {
            center: [2.0, 2.0, 2.0],
            radius: 5.0,
            intensity: 3.0,
            ..Default::default()
        };

        world.create_now()
            .with(light)
            .build();
    }

    fn handle_events(&mut self, events: &[WindowEvent], _: &mut Engine) -> Trans {
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
    let path = format!("{}/examples/02_sphere/resources/config.yml",
                       env!("CARGO_MANIFEST_DIR"));
    let cfg = DisplayConfig::from_file(path).unwrap();
    let mut game = Application::build(Example, cfg).done();
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
