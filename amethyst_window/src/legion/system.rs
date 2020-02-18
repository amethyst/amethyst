use crate::resources::ScreenDimensions;
use amethyst_config::Config;
use amethyst_core::{legion::prelude::*, shrev::EventChannel};
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

    pub fn build(
        world: &mut World,
        resources: &mut Resources,
        window: Window,
    ) -> Box<dyn Schedulable> {
        let (width, height) = window
            .get_inner_size()
            .expect("Window closed during initialization!")
            .into();

        let hidpi = window.get_hidpi_factor();

        resources.insert(ScreenDimensions::new(width, height, hidpi));

        resources.insert(window);

        SystemBuilder::<()>::new("WindowSystem")
            .write_resource::<ScreenDimensions>()
            .read_resource::<Window>()
            .build(move |_, _, (screen_dimensions, window), _| {
                manage_dimensions(&mut &mut *screen_dimensions, &window);
            })
    }
}

pub mod events_loop_system {
    //! System that polls the window events and pushes them to appropriate event channels.
    use super::*;

    /// Creates a new `EventsLoopSystem` using the provided `EventsLoop`
    pub fn build(
        world: &mut World,
        resources: &mut Resources,
        events_loop: EventsLoop,
    ) -> Box<dyn Runnable> {
        pub struct State {
            events_loop: EventsLoop,
            events: Vec<Event>,
        }
        let mut state = State {
            events_loop,
            events: Vec::with_capacity(128),
        };

        SystemBuilder::<()>::new("EventsLoopSystem")
            .write_resource::<EventChannel<Event>>()
            .build_thread_local(move |_, _, event_handler, _| {
                let events = &mut state.events;
                state.events_loop.poll_events(|event| {
                    events.push(event);
                });
                event_handler.drain_vec_write(events);
            })
    }
}
