use crate::resources::ScreenDimensions;
use amethyst_config::Config;
use amethyst_core::{
    legion::{
        schedule::Schedulable, system::SystemBuilder, SystemDesc, ThreadLocal, ThreadLocalDesc,
        World,
    },
    shrev::EventChannel,
};
use std::path::Path;
use winit::{Event, EventsLoop, Window};

pub mod window_system {
    use super::*;

    fn manage_dimensions(mut screen_dimensions: &mut ScreenDimensions, window: &Window) {
        let width = screen_dimensions.w;
        let height = screen_dimensions.h;

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            window.set_inner_size((width, height).into());
            screen_dimensions.dirty = false;
        }

        let hidpi = window.get_hidpi_factor();

        if let Some(size) = window.get_inner_size() {
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

    pub fn build(world: &mut World, window: Window) -> Box<dyn Schedulable> {
        let (width, height) = window
            .get_inner_size()
            .expect("Window closed during initialization!")
            .into();

        let hidpi = window.get_hidpi_factor();

        world
            .resources
            .insert(ScreenDimensions::new(width, height, hidpi));

        world.resources.insert(window);

        SystemBuilder::<()>::new("WindowSystem")
            .write_resource::<ScreenDimensions>()
            .read_resource::<Window>()
            .build(move |_, _, (screen_dimensions, window), _| {
                manage_dimensions(&mut &mut *screen_dimensions, &window);
            })
    }
}

/// System that polls the window events and pushes them to appropriate event channels.
///
/// This system must be active for any `GameState` to receive
/// any `StateEvent::Window` event into it's `handle_event` method.
#[derive(Debug)]
pub struct EventsLoopSystem {
    events_loop: EventsLoop,
    events: Vec<Event>,
}

impl EventsLoopSystem {
    /// Creates a new `EventsLoopSystem` using the provided `EventsLoop`
    pub fn new(events_loop: EventsLoop) -> Self {
        Self {
            events_loop,
            events: Vec::with_capacity(128),
        }
    }
}

impl ThreadLocal for EventsLoopSystem {
    fn run(&mut self, world: &mut World) {
        let mut event_handler = world.resources.get_mut::<EventChannel<Event>>().unwrap();

        let events = &mut self.events;
        self.events_loop.poll_events(|event| {
            events.push(event);
        });
        event_handler.drain_vec_write(events);
    }
    fn dispose(self, world: &mut World) {}
}
