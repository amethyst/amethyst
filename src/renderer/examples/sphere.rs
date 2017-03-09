//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate genmesh;
extern crate winit;

use genmesh::{MapToVertices, Triangulate, Vertices};
use genmesh::generators::SphereUV;
use std::time::{Duration, Instant};
use renderer::{RendererBuilder, Stage, Target};
use renderer::pass::ClearTarget;
use renderer::vertex::PosColor;
use winit::{Event, WindowBuilder};
use winit::ElementState::Pressed;
use winit::VirtualKeyCode as Key;

fn main() {
    let builder = WindowBuilder::new()
        .with_title("Amethyst Renderer Demo")
        .with_dimensions(1024, 768);

    let (window, mut renderer) = RendererBuilder::new(builder)
        .build()
        .expect("Could not build renderer");

    let pipe = renderer.create_pipeline()
        .with_target(Target::new("gbuffer")
            .with_num_color_bufs(4)
            .with_depth_buf(true))
        .with_stage(Stage::with_target("gbuffer")
            .with_pass(ClearTarget::with_values([1.0; 4], 1.0)))
        .with_stage(Stage::with_target("")
            .with_pass(ClearTarget::with_values([1.0; 4], 1.0))
            .enabled_by_default(true))
        .with_stage(Stage::with_target("")
            .with_pass(ClearTarget::with_values([0.0; 4], 1.0))
            .enabled_by_default(false))
        .build()
        .expect("Could not build pipeline");

    let mut delta = Duration::from_secs(0);

    let verts = gen_sphere(32, 32);
    let mesh = renderer.create_mesh(&verts).build();

    'main: loop {
        let start = Instant::now();

        for event in window.poll_events() {
            match event {
                Event::Closed | Event::KeyboardInput(Pressed, _, Some(Key::Escape)) => {
                    break 'main
                }
                _ => (),
            }
        }

        renderer.draw(&pipe, delta);
        window.swap_buffers().expect("Window error");

        let end = Instant::now();
        delta = end - start;
    }
}

fn gen_sphere(u: usize, v: usize) -> Vec<PosColor> {
    SphereUV::new(u, v)
        .vertex(|(x, y, z)| {
            PosColor {
                position: [x, y, z],
                color: [0.5, 1.0, 0.0, 1.0],
            }
        })
        .triangulate()
        .vertices()
        .collect()
}
