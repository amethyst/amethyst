//! Launches a new renderer window.

extern crate amethyst_renderer as renderer;
extern crate winit;

use renderer::prelude::*;
use std::time::{Duration, Instant};
use winit::{ControlFlow, Event, EventsLoop, WindowEvent};

fn main() {
    let mut events = EventsLoop::new();
    let mut renderer = Renderer::new(&events).unwrap();
    let pipe = renderer.create_pipe(Pipeline::forward()).unwrap();
    let scene = Scene::default();

    let mut delta = Duration::from_secs(0);
    events.run_forever(|e| {
        let start = Instant::now();

        match e {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { .. } | 
                WindowEvent::Closed => return ControlFlow::Break,
                _ => (),
            },
            _ => (),
        }

        renderer.draw(&scene, &pipe, delta);
        delta = Instant::now() - start;
        ControlFlow::Continue
    });
}
