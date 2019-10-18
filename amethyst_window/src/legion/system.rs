use crate::{config::DisplayConfig, resources::ScreenDimensions};
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

/// System for opening and managing the window.
#[derive(Debug)]
pub struct WindowSystem;

impl WindowSystem {
    /// Builds and spawns a new `Window`, using the provided `DisplayConfig` and `EventsLoop` as
    /// sources. Returns a new `WindowSystem`
    pub fn from_config_path(
        world: &mut World,
        events_loop: &EventsLoop,
        path: impl AsRef<Path>,
    ) -> Self {
        Self::from_config(world, events_loop, DisplayConfig::load(path.as_ref()))
    }

    /// Builds and spawns a new `Window`, using the provided `DisplayConfig` and `EventsLoop` as
    /// sources. Returns a new `WindowSystem`
    pub fn from_config(world: &mut World, events_loop: &EventsLoop, config: DisplayConfig) -> Self {
        let window = config
            .into_window_builder(events_loop)
            .build(events_loop)
            .unwrap();
        Self::new(world, window)
    }

    /// Create a new `WindowSystem` wrapping the provided `Window`
    pub fn new(world: &mut World, window: Window) -> Self {
        let (width, height) = window
            .get_inner_size()
            .expect("Window closed during initialization!")
            .into();

        let hidpi = window.get_hidpi_factor();

        world
            .resources
            .insert(ScreenDimensions::new(width, height, hidpi));

        world.resources.insert(window);

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
}

pub struct WindowSystemDesc {
    pub system: WindowSystem,
}
impl WindowSystemDesc {
    pub fn new(system: WindowSystem) -> Self {
        Self { system }
    }
}
impl SystemDesc for WindowSystemDesc {
    fn build(self, world: &mut World) -> Box<dyn Schedulable> {
        SystemBuilder::<()>::new("WindowSystem")
            .write_resource::<ScreenDimensions>()
            .read_resource::<Window>()
            .build_disposable(
                self.system,
                |state, _, _, (screen_dimensions, window), _| {
                    state.manage_dimensions(&mut &mut *screen_dimensions, &window);
                },
                |state, world| {},
            )
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
