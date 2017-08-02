//! Rendering system.

// use config::Config;
use ecs::{System, World};
use event::EventsIter;
use error::{Error, Result};
use renderer::prelude::*;
use super::SystemExt;
use winit::EventsLoop;

/// Rendering system.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderSystem {
    #[derivative(Debug = "ignore")]
    events: EventsLoop,
    pipe: Pipeline,
    #[derivative(Debug = "ignore")]
    renderer: Renderer,
    scene: Scene,
}

impl<'a> SystemExt<'a> for RenderSystem {
    fn build(_: ()) -> Result<RenderSystem> {
        let events = EventsLoop::new();
        let mut renderer = Renderer::new(&events).map_err(|_| Error::System)?;
        let pipe = renderer.create_pipe(Pipeline::forward()).map_err(|_| Error::System)?;

        Ok(RenderSystem {
            events: events,
            pipe: pipe,
            renderer: renderer,
            scene: Scene::default(),
        })
    }

    fn register(_world: &mut World) {

    }

    fn poll_events(&mut self) -> EventsIter {
        let mut new_events = Vec::new();
        self.events.poll_events(|e| {
            new_events.push(e);
        });

        EventsIter::from(new_events)
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = ();
    fn run(&mut self, mut data: Self::SystemData) {
        use std::time::Duration;
        self.renderer.draw(&self.scene, &self.pipe, Duration::from_secs(0));
    }
}
