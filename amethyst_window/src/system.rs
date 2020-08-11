use crate::resources::ScreenDimensions;
use amethyst_core::{ecs::*, EventChannel};
use winit::{Event, EventsLoop, Window};

/// Updates `ScreenDimensions` struct with the actual window size from `Window`.
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

/// Builds window system that updates `ScreenDimensions` resource from a provided `Window`.
pub fn build_window_system() -> impl Runnable {
    SystemBuilder::new("WindowSystem")
        .write_resource::<ScreenDimensions>()
        .read_resource::<Window>()
        .build(
            |_commands, _world, (screen_dimensions, window_res), _query| {
                manage_dimensions(screen_dimensions, window_res)
            },
        )
}

/// System that polls the window events and pushes them to appropriate event channels.
///
/// This system must be active for any `GameState` to receive
/// any `StateEvent::Window` event into it's `handle_event` method.
pub fn build_events_loop_system(mut events_loop: EventsLoop) -> impl Runnable {
    let mut events = Vec::with_capacity(128);
    SystemBuilder::new("EventsLoopSystem")
        .write_resource::<EventChannel<Event>>()
        .build(move |_commands, _world, event_channel, _query| {
            events_loop.poll_events(|event| {
                events.push(event);
            });
            event_channel.drain_vec_write(&mut events);
        })
}
