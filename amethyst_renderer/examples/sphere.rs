//! Launches a new renderer window.

#[macro_use] extern crate error_chain;
extern crate amethyst_renderer as renderer;
extern crate cgmath;
extern crate genmesh;
extern crate winit;

use cgmath::{Deg, Matrix4, Vector3};
use cgmath::prelude::{InnerSpace, Transform};
use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;
use renderer::prelude::*;
use renderer::vertex::PosNormTex;
use renderer::Result;

fn run() -> Result<()> {
    use std::time::{Duration, Instant};
    use winit::{Event, EventsLoop, WindowEvent};

    let mut events = EventsLoop::new();
    let mut renderer = Renderer::new(&events)?;
    let pipe = renderer
        .create_pipe(
            Pipeline::build().with_stage(
                Stage::with_backbuffer()
                    .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
                    .with_model_pass(pass::DrawFlat::<PosNormTex>::new()),
            ),
        )?;

    let verts = gen_sphere(32, 32);
    let mesh = renderer.create_mesh(Mesh::build(&verts))?;

    let tex = Texture::from_color_val([0.88235, 0.09412, 0.21569, 1.0]);
    let mtl = renderer
        .create_material(MaterialBuilder::new().with_albedo(tex))?;
    let model = Model {
        mesh: mesh,
        material: mtl,
        pos: Matrix4::one(),
    };

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
    let mut running = true;
    while running {
        let start = Instant::now();

        events.poll_events(|e| match e {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput { .. } |
                    WindowEvent::Closed => running = false,
                    _ => (),
                }
            }
            _ => (),
        });

        renderer.draw(&scene, &pipe, delta)?;
        delta = Instant::now() - start;
    }
    Ok(())
}

quick_main!(run);

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
