//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate winit;

use std::time::{Duration, Instant};
use renderer::{Renderer, Scene, Stage};
use renderer::pass::ClearTarget;
use winit::{Event, WindowBuilder};
use winit::ElementState::Pressed;
use winit::VirtualKeyCode as Key;

fn main() {
    let builder = WindowBuilder::new()
        .with_title("Amethyst Renderer Demo")
        .with_dimensions(1024, 768);

    let mut renderer = Renderer::from_winit_builder(builder)
        .expect("Could not build renderer");

    let pipe = renderer.create_pipeline()
        .with_stage(Stage::with_target("")
            .with_pass(ClearTarget::with_values([0.0, 0.0, 0.0, 1.0], 1.0)))
        .build()
        .expect("Could not build pipeline");

    let scene = Scene::default();

    let mut delta = Duration::from_secs(0);

    'main: loop {
        let start = Instant::now();

        for event in renderer.window().poll_events() {
            match event {
                Event::Closed | Event::KeyboardInput(Pressed, _, Some(Key::Escape)) => {
                    break 'main
                }
                _ => (),
            }
        }

        renderer.draw(&scene, &pipe, delta);

        delta = Instant::now() - start;
    }
}
