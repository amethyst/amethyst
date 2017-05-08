//! Rendering system.

use config::Config;
use ecs::{RunArg, System, World};
use engine::EventsIter;
use error::Result;
use renderer::prelude::*;
use super::SystemExt;
use winit::EventsLoop;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderingSystem {
    #[derivative(Debug = "ignore")]
    events: EventsLoop,
    // renderer: Renderer,
    scene: Scene,
}

impl SystemExt for RenderingSystem {
    fn build(_: &Config) -> Result<RenderingSystem> {
        let events = EventsLoop::new();
        // let renderer = Renderer::new(&events)?;
        Ok(RenderingSystem {
            events: events,
            // renderer: Mutex::new(renderer),
            scene: Scene::default(),
        })
    }

    fn register(_world: &mut World) {

    }

    fn poll_events(&self) -> EventsIter {
        let mut new_events = Vec::new();
        self.events.poll_events(|e| {
            new_events.push(e);
        });

        EventsIter::from(new_events)
    }
}

impl System<()> for RenderingSystem {
    fn run(&mut self, arg: RunArg, _: ()) {
        use ecs::Gate;
    }
}
