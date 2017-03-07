//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate winit;

use std::time::{Duration, Instant};
use renderer::{RendererBuilder, Stage, Target};
use renderer::pass::ClearTarget;
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

    'main: loop {
        let start = Instant::now();

        for event in window.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(Pressed, _, Some(Key::Escape)) => break 'main,
                _ => (),
            }
        }

        renderer.draw(&pipe, delta);
        window.swap_buffers().expect("Window error");

        let end = Instant::now();
        delta = end - start;
    }
}
