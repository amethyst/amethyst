//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate glutin;

use renderer::prelude::*;
use std::time::{Duration, Instant};
use glutin::{Event, EventsLoop, WindowEvent};

fn main() {
    let mut events = EventsLoop::new();
    let mut renderer = Renderer::new(&events).unwrap();
    let pipe = renderer.create_pipe(Pipeline::forward()).unwrap();
    let scene = Scene::default();

    let mut delta = Duration::from_secs(0);
    let mut running = true;
    while running {
        let start = Instant::now();

        events.poll_events(|e| {
            match e {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { .. } |
                    WindowEvent::Closed => running = false,
                    _ => (),
                },
                _ => (),
            }
        });

        renderer.draw(&scene, &pipe, delta);
        delta = Instant::now() - start;
    }
}
