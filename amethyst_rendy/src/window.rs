use amethyst_core::{
    shrev::EventChannel,
    ecs::{Write, RunNow, Resources, SystemData}
};
use amethyst_config::Config;
use winit::{Event, EventsLoop, Window};
use crate::config::DisplayConfig;
use std::path::Path;
use std::sync::Arc;

// TODO: move out of example code
pub struct WindowSystem {
    window: Arc<Window>,
}

impl WindowSystem {
    pub fn from_config_path(event_loop: &EventsLoop, path: impl AsRef<Path>) -> Self {
        Self::from_config(event_loop, DisplayConfig::load(path.as_ref()))
    }

    pub fn from_config(event_loop: &EventsLoop, config: DisplayConfig) -> Self {
        let window = config
            .to_windowbuilder(event_loop)
            .build(event_loop)
            .unwrap();
        Self::new(window)
    }

    pub fn new(window: Window) -> Self {
        Self { 
            window: Arc::new(window),
        }
    }
}

impl<'a> RunNow<'a> for WindowSystem {
    fn run_now(&mut self, _res: &'a Resources) {}
    fn setup(&mut self, res: &mut Resources) {
        res.insert(self.window.clone());
    }
}

pub struct EventPollingSystem {
    event_loop: EventsLoop,
}

impl EventPollingSystem {
    pub fn new(event_loop: EventsLoop) -> Self {
        Self { event_loop }
    }
}

impl<'a> RunNow<'a> for EventPollingSystem {
    fn run_now(&mut self, res: &'a Resources) {
        let mut event_handler = <Write<'a, EventChannel<Event>>>::fetch(res);

        self.event_loop.poll_events(|event| {
            event_handler.single_write(event);
        });
    }

    fn setup(&mut self, res: &mut Resources) {
        <Write<'a, EventChannel<Event>>>::setup(res);
    }
}
