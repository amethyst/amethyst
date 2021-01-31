use amethyst_core::{
    dispatcher::{System, ThreadLocalSystem},
    ecs::{systems::ParallelRunnable, Runnable, SystemBuilder},
    EventChannel,
};
use winit::{
    dpi::Size,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::Window,
};

use crate::resources::ScreenDimensions;

/// Manages window dimensions
#[derive(Debug)]
pub struct WindowSystem;

/// Builds window system that updates `ScreenDimensions` resource from a provided `Window`.
impl System for WindowSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("WindowSystem")
                .write_resource::<ScreenDimensions>()
                .read_resource::<Window>()
                .build(|_commands, _world, (screen_dimensions, window), _query| {
                    let width = screen_dimensions.w;
                    let height = screen_dimensions.h;

                    // Send resource size changes to the window
                    if screen_dimensions.dirty {
                        window.set_inner_size(Size::Logical((width, height).into()));
                        screen_dimensions.dirty = false;
                    }

                    let (window_width, window_height): (f64, f64) = window.inner_size().into();

                    // Send window size changes to the resource
                    if (window_width, window_height) != (width, height) {
                        screen_dimensions.update(window_width, window_height);

                        // We don't need to send the updated size of the window back to the window itself,
                        // so set dirty to false.
                        screen_dimensions.dirty = false;
                    }
                }),
        )
    }
}
/// System that polls the window events and pushes them to appropriate event channels.
///
/// This system must be active for any `GameState` to receive
/// any `StateEvent::Window` event into it's `handle_event` method.
#[derive(Debug)]
pub struct EventLoopSystem {
    pub(crate) event_loop: EventLoop<()>,
}

impl ThreadLocalSystem<'static> for EventLoopSystem {
    fn build(mut self) -> Box<dyn Runnable> {
        let mut events = Vec::with_capacity(128);

        Box::new(
            SystemBuilder::new("EventsLoopSystem")
                .write_resource::<EventChannel<Event<'static, ()>>>()
                .build(move |_commands, _world, event_channel, _query| {
                    self.event_loop.run_return(|event, _, flow| {
                        match event {
                            Event::WindowEvent { .. } => {
                                events.push(event.to_static().unwrap());
                            }
                            Event::DeviceEvent { .. } => {
                                events.push(event.to_static().unwrap());
                            }
                            _ => {}
                        }
                        *flow = ControlFlow::Exit;
                    });
                    event_channel.drain_vec_write(&mut events);
                }),
        )
    }
}
