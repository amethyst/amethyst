//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate glutin;
extern crate winit;

use glutin::EventsLoop;
use renderer::prelude::*;
use std::time::{Duration, Instant};
use winit::ElementState::Pressed;
use winit::{Event, WindowEvent};
use winit::VirtualKeyCode as Key;

fn main() {
    let events = EventsLoop::new();
    let mut renderer = Renderer::new(&events).unwrap();
    let pipe = renderer.create_pipe(Pipeline::forward()).unwrap();
    let scene = Scene::default();

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
