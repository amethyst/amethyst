extern crate amethyst;
extern crate genmesh;
extern crate cgmath;

use genmesh::generators::SphereUV;
use genmesh::{Triangulate, MapToVertices, Vertices};
use cgmath::{Vector3, EuclideanVector};

use amethyst::engine::{Application, State, Trans};
use amethyst::context::Context;
use amethyst::config::Element;
use amethyst::ecs::{World, Entity};
use amethyst::renderer::VertexPosNormal;

struct Example;

impl State for Example {
    fn handle_events(&mut self, events: Vec<Entity>, context: &mut Context, _: &mut World) -> Trans {
        use amethyst::context::event::{EngineEvent, Event, VirtualKeyCode};
        let mut trans = Trans::None;
        let storage = context.broadcaster.read::<EngineEvent>();
        for _event in events {
            let event = storage.get(_event).unwrap();
            let event = &event.payload;
            match *event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => trans = Trans::Quit,
                Event::Closed => trans = Trans::Quit,
                _ => (),
            }
        }
        trans
    }

    fn on_start(&mut self, context: &mut Context, _: &mut World) {
        use amethyst::renderer::pass::{Clear, DrawFlat};
        use amethyst::renderer::{Layer, Camera};
        let (w, h) = context.renderer.get_dimensions().unwrap();
        let proj = Camera::perspective(60.0, w as f32 / h as f32, 1.0, 100.0);
        let eye = [0., 10., 0.];
        let target = [0., 0., 0.];
        let up = [0., 0., 1.];
        let view = Camera::look_at(eye, target, up);
        let camera = Camera::new(proj, view);

        context.renderer.add_scene("main");
        context.renderer.add_camera(camera, "main");

        let sphere = build_sphere();

        let ka = context.renderer.create_constant_texture([1., 1., 1., 1.]);
        let kd = context.renderer.create_constant_texture([1., 1., 1., 1.]);

        let transform: [[f32; 4]; 4] = cgmath::Matrix4::from_scale(1.).into();

        let fragment = context.renderer.create_fragment(&sphere, ka, kd, transform).unwrap();
        context.renderer.add_fragment("main", fragment);

        let layer =
            Layer::new("main",
                        vec![
                            Clear::new([0., 0., 0., 1.]),
                            DrawFlat::new("main", "main"),
                        ]);
        let pipeline = vec![layer];
        context.renderer.set_pipeline(pipeline);
    }

    fn update(&mut self, context: &mut Context, _: &mut World) -> Trans {
        context.renderer.submit();
        Trans::None
    }
}

fn main() {
    use amethyst::context::Config;
    let config = Config::from_file("../config/window_example_config.yml").unwrap();
    let mut game = Application::build(Example, config).done();
    game.run();
}

fn build_sphere() -> Vec<VertexPosNormal> {
    SphereUV::new(32, 32)
        .vertex(|(x, y, z)| VertexPosNormal {
            pos: [x, y, z],
            normal: Vector3::new(x, y, z).normalize().into(),
            tex_coord: [0., 0.]
        })
        .triangulate()
        .vertices()
        .collect()
}
