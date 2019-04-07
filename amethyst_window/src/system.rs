use crate::{config::DisplayConfig, resources::ScreenDimensions};
use amethyst_config::Config;
use amethyst_core::{
    ecs::{Resources, RunNow, System, SystemData, Write, WriteExpect},
    shrev::EventChannel,
};
use std::{path::Path, sync::Arc};
use winit::{Event, EventsLoop, Window};

/// System for opening and managing the window.
pub struct WindowSystem {
    window: Arc<Window>,
}

impl WindowSystem {
    pub fn from_config_path(events_loop: &EventsLoop, path: impl AsRef<Path>) -> Self {
        Self::from_config(events_loop, DisplayConfig::load(path.as_ref()))
    }

    pub fn from_config(events_loop: &EventsLoop, config: DisplayConfig) -> Self {
        let window = config
            .to_windowbuilder(events_loop)
            .build(events_loop)
            .unwrap();
        Self::new(window)
    }

    pub fn new(window: Window) -> Self {
        Self {
            window: Arc::new(window),
        }
    }

    fn manage_dimensions(&mut self, mut screen_dimensions: &mut ScreenDimensions) {
        let width = screen_dimensions.w;
        let height = screen_dimensions.h;

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            self.window.set_inner_size((width, height).into());
            screen_dimensions.dirty = false;
        }

        let hidpi = self.window.get_hidpi_factor();

        if let Some(size) = self.window.get_inner_size() {
            let (window_width, window_height): (f64, f64) = size.to_physical(hidpi).into();

            // Send window size changes to the resource
            if (window_width, window_height) != (width, height) {
                screen_dimensions.update(window_width, window_height);

                // We don't need to send the updated size of the window back to the window itself,
                // so set dirty to false.
                screen_dimensions.dirty = false;
            }
        }
        screen_dimensions.update_hidpi_factor(hidpi);
    }
}

impl<'a> System<'a> for WindowSystem {
    type SystemData = WriteExpect<'a, ScreenDimensions>;

    fn run(&mut self, mut screen_dimesnions: Self::SystemData) {
        self.manage_dimensions(&mut screen_dimesnions);
    }
    fn setup(&mut self, res: &mut Resources) {
        let (width, height) = self
            .window
            .get_inner_size()
            .expect("Window closed during initialization!")
            .into();
        let hidpi = self.window.get_hidpi_factor();
        res.insert(ScreenDimensions::new(width, height, hidpi));
        res.insert(self.window.clone());
    }
}

/// System that polls the window events and pushes them to appropriate event channels.
///
/// This system must be active for any `GameState` to receive
/// any `StateEvent::Window` event into it's `handle_event` method.
pub struct EventsLoopSystem {
    events_loop: EventsLoop,
    events: Vec<Event>,
}

impl EventsLoopSystem {
    pub fn new(events_loop: EventsLoop) -> Self {
        Self {
            events_loop,
            events: Vec::with_capacity(128),
        }
    }
}

impl<'a> RunNow<'a> for EventsLoopSystem {
    fn run_now(&mut self, res: &'a Resources) {
        let mut event_handler = <Write<'a, EventChannel<Event>>>::fetch(res);

        let events = &mut self.events;
        self.events_loop.poll_events(|event| {
            events.push(event);
        });
        event_handler.drain_vec_write(events);
    }

    fn setup(&mut self, res: &mut Resources) {
        <Write<'a, EventChannel<Event>>>::setup(res);
    }
}
