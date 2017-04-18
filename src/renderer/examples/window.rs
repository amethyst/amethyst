//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate winit;

use renderer::prelude::*;
use std::time::{Duration, Instant};
use winit::ElementState::Pressed;
use winit::Event;
use winit::VirtualKeyCode as Key;

fn main() {
    let mut renderer = Renderer::new().unwrap();
    let pipe = renderer.create_pipe(pipe::forward()).unwrap();
    let scene = Scene::default();

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
