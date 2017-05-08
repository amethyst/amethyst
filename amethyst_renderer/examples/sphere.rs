//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate genmesh;
extern crate winit;
extern crate glutin;

use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;
use glutin::EventsLoop;
use renderer::prelude::*;
use renderer::vertex::PosColor;
use std::time::{Duration, Instant};
use winit::ElementState::Pressed;
use winit::{Event, WindowEvent};
use winit::VirtualKeyCode as Key;

fn main() {
    let events = EventsLoop::new();
    let mut renderer = Renderer::new(&events).unwrap();
    let pipe = renderer.create_pipe(Pipeline::forward()).unwrap();

    let verts = gen_sphere(32, 32);
    let mesh = renderer.create_mesh(Mesh::new(&verts)).unwrap();

    let mut scene = Scene::default();
    scene.add_mesh("ball", mesh);
    scene.add_light("lamp", PointLight::default());

    let mut running = true;
    let mut delta = Duration::from_secs(0);
    while running {
        let start = Instant::now();
        events.poll_events(|e| {
            let Event::WindowEvent { event, .. } = e;
            match event {
                WindowEvent::Closed |
                WindowEvent::KeyboardInput(Pressed, _, Some(Key::Escape), _) => running = false,
                _ => (),
            }
        });

        renderer.draw(&scene, &pipe, delta);
        delta = Instant::now() - start;
    }
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosColor> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            PosColor {
                a_position: [x, y, z],
                a_color: [0.5, 1.0, 0.0, 1.0],
            }
        })
        .triangulate()
        .vertices()
        .collect()
}
