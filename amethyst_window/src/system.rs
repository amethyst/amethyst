use amethyst_core::{
    dispatcher::{System, ThreadLocalSystem},
    ecs::{systems::ParallelRunnable, Runnable, SystemBuilder},
    EventChannel,
};
use winit::{Event, EventsLoop, Window};

use crate::resources::ScreenDimensions;

/// Manages window dimensions
#[derive(Debug)]
pub struct WindowSystem;

/// Builds window system that updates `ScreenDimensions` resource from a provided `Window`.
impl System<'_> for WindowSystem {
    fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("WindowSystem")
                .write_resource::<ScreenDimensions>()
                .read_resource::<Window>()
                .build(|_commands, _world, (screen_dimensions, window), _query| {
                    let width = screen_dimensions.w;
                    let height = screen_dimensions.h;

                    // Send resource size changes to the window
                    if screen_dimensions.dirty {
                        window.set_inner_size((width, height).into());
                        screen_dimensions.dirty = false;
                    }

                    let hidpi = window.get_hidpi_factor();

                    if let Some(size) = window.get_inner_size() {
                        let (window_width, window_height): (f64, f64) =
                            size.to_physical(hidpi).into();

                        // Send window size changes to the resource
                        if (window_width, window_height) != (width, height) {
                            screen_dimensions.update(window_width, window_height);

                            // We don't need to send the updated size of the window back to the window itself,
                            // so set dirty to false.
                            screen_dimensions.dirty = false;
                        }
                    }
                    screen_dimensions.update_hidpi_factor(hidpi);
                }),
        )
    }
}

/// reports new window events from winit to the EventChannel
#[derive(Debug)]
pub struct WindowEventsSystem {
    /// winit EventsLoop for window events
    pub events_loop: EventsLoop,
}

/// System that polls the window events and pushes them to appropriate event channels.
///
/// This system must be active for any `GameState` to receive
/// any `StateEvent::Window` event into it's `handle_event` method.
impl ThreadLocalSystem<'static> for WindowEventsSystem {
    fn build(&'static mut self) -> Box<dyn Runnable> {
        let mut events = Vec::with_capacity(128);

        Box::new(
            SystemBuilder::new("EventsLoopSystem")
                .write_resource::<EventChannel<Event>>()
                .build(move |_commands, _world, event_channel, _query| {
                    self.events_loop.poll_events(|event| {
                        events.push(event);
                    });
                    event_channel.drain_vec_write(&mut events);
                }),
        )
    }
}
