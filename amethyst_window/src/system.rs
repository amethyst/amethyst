use crate::{config::DisplayConfig, resources::ScreenDimensions};
use amethyst_config::Config;
use amethyst_core::{
    ecs::{ReadExpect, Resources, RunNow, System, SystemData, Write, WriteExpect},
    shrev::EventChannel,
    WindowRes,
};
use std::path::Path;
use winit::{event::Event, event_loop::EventLoop, window::Window};

/// System for opening and managing the window.
pub struct WindowSystem {
    create: Option<Box<dyn FnOnce(&mut Resources) + Send + Sync>>,
}

impl WindowSystem {
    pub(crate) fn from_closure(f: Box<dyn FnOnce(&mut Resources) + Send + Sync>) -> Self {
        WindowSystem {
            create: Some(f),
        }
    }

    fn manage_dimensions(mut screen_dimensions: &mut ScreenDimensions, window: &Window) {
        let width = screen_dimensions.w;
        let height = screen_dimensions.h;

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            window.set_inner_size((width, height).into());
            screen_dimensions.dirty = false;
        }

        let hidpi = window.hidpi_factor();
        let size = window.inner_size();

        let (window_width, window_height): (f64, f64) = size.to_physical(hidpi).into();
        // Send window size changes to the resource
        if (window_width, window_height) != (width, height) {
            screen_dimensions.update(window_width, window_height);

            // We don't need to send the updated size of the window back to the window itself,
            // so set dirty to false.
            screen_dimensions.dirty = false;
        }
        screen_dimensions.update_hidpi_factor(hidpi);
    }
}

impl<'a> System<'a> for WindowSystem {
    type SystemData = (WriteExpect<'a, ScreenDimensions>, ReadExpect<'a, WindowRes>);

    fn run(&mut self, (mut screen_dimensions, window): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("window_system");

        Self::manage_dimensions(&mut screen_dimensions, &window);
    }

    fn setup(&mut self, res: &mut Resources) {
        (self.create.take().unwrap())(res);
        let window = res.fetch::<WindowRes>();
        let (width, height) = window
            .inner_size()
            .into();
        let hidpi = window.hidpi_factor();
        drop(window);
        res.insert(ScreenDimensions::new(width, height, hidpi));
    }
}

// /// System that polls the window events and pushes them to appropriate event channels.
// ///
// /// This system must be active for any `GameState` to receive
// /// any `StateEvent::Window` event into it's `handle_event` method.
// #[derive(Debug)]
// pub struct EventsLoopSystem {
//     events_loop: EventLoop,
//     events: Vec<Event>,
// }

// impl EventsLoopSystem {
//     /// Creates a new `EventsLoopSystem` using the provided `EventLoop`
//     pub fn new(events_loop: EventLoop) -> Self {
//         Self {
//             events_loop,
//             events: Vec::with_capacity(128),
//         }
//     }
// }

// impl<'a> RunNow<'a> for EventsLoopSystem {
//     fn run_now(&mut self, res: &'a Resources) {
//         let mut event_handler = <Write<'a, EventChannel<Event>>>::fetch(res);

//         let events = &mut self.events;
//         self.events_loop.poll_events(|event| {
//             events.push(event);
//         });
//         event_handler.drain_vec_write(events);
//     }

//     fn setup(&mut self, res: &mut Resources) {
//         <Write<'a, EventChannel<Event>>>::setup(res);
//     }
// }
