//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate genmesh;
extern crate winit;
extern crate glutin;
extern crate cgmath;

use cgmath::{Matrix4, Deg, Vector3};
use cgmath::prelude::{InnerSpace, Transform};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;
use glutin::EventsLoop;
use renderer::prelude::*;
use renderer::vertex::PosNormTex;
use std::time::{Duration, Instant};
use winit::ElementState::Pressed;
use winit::{Event, WindowEvent};
use winit::VirtualKeyCode as Key;

fn main() {
    let events = EventsLoop::new();
    let mut renderer = Renderer::new(&events).expect("Renderer create");
    let pipe = renderer.create_pipe(
        Pipeline::forward()
            .with_stage(
                Stage::with_backbuffer()
                    .with_pass(pass::ClearTarget::with_values([0.00196, 0.23726, 0.21765, 1.0], Some(1.0f32)))
                    .with_pass(&pass::DrawFlat::<PosNormTex>::new())
            )
    ).expect("Pipeline create");

    let verts = gen_sphere(32, 32);
    let mesh = renderer.create_mesh(Mesh::build(&verts)).expect("Mesh create");

   // let bytes = load_texture("bricks.png").unwrap();
    // let tex = renderer.create_texture(Texture::build(&bytes)).unwrap();
    let tex = renderer.create_texture(Texture::from_color_val([0.88235, 0.09412, 0.21569, 1.0])).expect("Texture create");
    let mtl = renderer.create_material(MaterialBuilder::new().with_albedo(&tex)).expect("Material create");
    let model = Model { mesh: mesh, material: mtl, pos: Matrix4::one() };

    let mut scene = Scene::default();
    scene.add_model(model);
    scene.add_light(PointLight::default());
    scene.add_camera(Camera {
        eye: [0.0, 0.0, -4.0].into(),
        proj: Projection::perspective(1.3, Deg(60.0)).into(),
        forward: [-0.1, 0.0, 1.0].into(),
        right: [1.0, 0.0, 0.0].into(),
        up: [0.0, 1.0, 0.0].into(),
    });

    let mut delta = Duration::from_secs(0);
    events.run_forever(|e| {
        let start = Instant::now();

        let Event::WindowEvent { event, .. } = e;
        match event {
            WindowEvent::KeyboardInput(Pressed, _, Some(Key::Escape), _) |
            WindowEvent::Closed => events.interrupt(),
            _ => (),
        }

        renderer.draw(&scene, &pipe, delta);
        delta = Instant::now() - start;
    });
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosNormTex> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            PosNormTex {
                a_position: [x, y, z],
                a_normal: Vector3::from([x, y, z]).normalize().into(),
                a_tex_coord: [0.1, 0.1],
            }
        })
        .triangulate()
        .vertices()
        .collect()
}
