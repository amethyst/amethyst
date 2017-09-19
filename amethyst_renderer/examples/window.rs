//! Launches a new renderer window.

#[macro_use] extern crate error_chain;
extern crate amethyst_renderer as renderer;
extern crate winit;

use std::time::{Duration, Instant};

use winit::{Event, EventsLoop, WindowEvent};
use renderer::prelude::*;
use renderer::Result;

fn run() -> Result<()> {
    let mut events = EventsLoop::new();
    let mut renderer = Renderer::new(&events)?;
    let pipe = renderer.create_pipe(Pipeline::forward::<PosTex>())?;
    let scene = Scene::default();

    let mut delta = Duration::from_secs(0);
    let mut running = true;
    while running {
        let start = Instant::now();

        events.poll_events(|e| match e {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::KeyboardInput { .. } |
                    WindowEvent::Closed => running = false,
                    _ => (),
                }
            }
            _ => (),
        });

        renderer.draw(&scene, &pipe, delta)?;
        delta = Instant::now() - start;
    }
    Ok(())
}

quick_main!(run);
