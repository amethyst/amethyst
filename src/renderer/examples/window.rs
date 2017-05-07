//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
//extern crate winit;
extern crate glutin;

use renderer::prelude::*;
use std::time::{Duration, Instant};
// use winit::ElementState::Pressed;
// use winit::{Event, EventsLoop, WindowEvent};
// use winit::VirtualKeyCode as Key;
use glutin::ElementState::Pressed;
use glutin::{Event, EventsLoop, WindowEvent};
use glutin::VirtualKeyCode as Key;

fn main() {
    let events = EventsLoop::new();
    let mut renderer = Renderer::new(&events).unwrap();
    let pipe = renderer.create_pipe(Pipeline::forward()).unwrap();
    let scene = Scene::default();

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
