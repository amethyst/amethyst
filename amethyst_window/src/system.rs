use crate::resources::ScreenDimensions;
use amethyst_core::{
    ecs::{ReadExpect, System, World, WriteExpect},
    shrev::EventChannel,
};
use std::sync::mpsc::{Receiver, TryRecvError};
use winit::{event::Event, window::Window};

/// System for opening and managing the window.
#[derive(Debug)]
pub struct WindowSystem;

impl WindowSystem {
    /// Create a new `WindowSystem` wrapping the provided `Window`
    pub fn new(world: &mut World, window: Window) -> Self {
        let (width, height) = window.inner_size().into();
        let hidpi = window.hidpi_factor();
        world.insert(ScreenDimensions::new(width, height, hidpi));
        world.insert(window);
        Self
    }

    fn manage_dimensions(&mut self, mut screen_dimensions: &mut ScreenDimensions, window: &Window) {
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
    type SystemData = (WriteExpect<'a, ScreenDimensions>, ReadExpect<'a, Window>);

    fn run(&mut self, (mut screen_dimensions, window): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("window_system");

        self.manage_dimensions(&mut screen_dimensions, &window);
    }
}

/// System that polls the window events and pushes them to appropriate event channels.
///
/// This system must be active for any `GameState` to receive
/// any `StateEvent::Window` event into it's `handle_event` method.
#[derive(Debug)]
pub struct EventLoopSystem {
    event_receiver: Receiver<Event<()>>,
    events: Vec<Event<()>>,
}

impl EventLoopSystem {
    /// Creates a new `EventsLoopSystem` using the provided `Receiver<Event<()>>`
    pub fn new(event_receiver: Receiver<Event<()>>) -> Self {
        Self {
            event_receiver,
            events: Vec::with_capacity(128),
        }
    }
}

impl<'a> System<'a> for EventLoopSystem {
    type SystemData = (WriteExpect<'a, EventChannel<Event<()>>>);

    fn run(&mut self, mut event_channel: Self::SystemData) {
        let events = &mut self.events;
        loop {
            match self.event_receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("Event loop crashed"),
            }
        }
        event_channel.drain_vec_write(events);
    }
}
