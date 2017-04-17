//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate genmesh;
extern crate winit;

use std::time::{Duration, Instant};

use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;
use renderer::prelude::*;
use renderer::pipe;
use renderer::vertex::PosColor;
use winit::ElementState::Pressed;
use winit::Event;
use winit::VirtualKeyCode as Key;

fn main() {
    let mut renderer = Renderer::new().unwrap();
    let pipe = pipe::forward(&mut renderer).unwrap();

    let verts = gen_sphere(32, 32);
    let mesh = renderer.create_mesh(&verts).build().unwrap();

    let mut scene = Scene::default();
    scene.add_mesh("ball", mesh);
    scene.add_light("lamp", PointLight::default());

    let mut delta = Duration::from_secs(0);
    'main: loop {
        let start = Instant::now();
        for event in renderer.window().poll_events() {
            match event {
                Event::Closed |
                Event::KeyboardInput(Pressed, _, Some(Key::Escape)) => break 'main,
                _ => (),
            }
        }

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
