use crate::{DisplayConfig, EventLoopSystem, WindowSystem};
use amethyst_config::Config;
use amethyst_core::{bundle::SystemBundle, ecs::World, shred::DispatcherBuilder};
use amethyst_error::Error;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};
use winit::{event::Event, event_loop::EventLoop, window::Window};

/// Screen width used in predefined display configuration.
#[cfg(feature = "test-support")]
pub const SCREEN_WIDTH: u32 = 800;
/// Screen height used in predefined display configuration.
#[cfg(feature = "test-support")]
pub const SCREEN_HEIGHT: u32 = 600;

/// Bundle providing easy initializing of the appopriate `Window`, `WindowSystem` `EventLoop` and
/// `EventLoopSystem` constructs used for creating the rendering window of amethyst with `winit`
#[derive(Debug)]
pub struct WindowBundle {
    config: DisplayConfig,
}

impl WindowBundle {
    /// Builds a new window bundle from a loaded `DisplayConfig`.
    pub fn from_config(config: DisplayConfig) -> Self {
        WindowBundle { config }
    }

    /// Builds a new window bundle by loading the `DisplayConfig` from `path`.
    ///
    /// Will fall back to `DisplayConfig::default()` in case of an error.
    pub fn from_config_path(path: impl AsRef<std::path::Path>) -> Self {
        WindowBundle::from_config(DisplayConfig::load(path.as_ref()))
    }

    /// Builds a new window bundle with a predefined `DisplayConfig`.
    ///
    /// This uses a `DisplayConfig::default()`, but with the following differences:
    ///
    /// * `dimensions` is changed to `Some((SCREEN_WIDTH, SCREEN_HEIGHT))`.
    /// * `visibility` is `false`.
    #[cfg(feature = "test-support")]
    pub fn from_test_config() -> Self {
        let mut display_config = DisplayConfig::default();
        display_config.dimensions = Some((SCREEN_WIDTH, SCREEN_HEIGHT));
        display_config.visibility = false;

        WindowBundle::from_config(display_config)
    }

    fn build_event_loop_and_window(self) -> (Receiver<Event<()>>, Window) {
        let (sender, receiver) = mpsc::channel::<Event<()>>();

        let window_return: Arc<Mutex<Option<Window>>> = Arc::new(Mutex::new(None));
        let done = Arc::new(AtomicBool::from(false));

        Self::spawn_event_loop(self.config, sender, window_return.clone(), done.clone());

        while !done.load(Ordering::SeqCst) {}

        let window = window_return.lock().unwrap().take().unwrap();

        (receiver, window)
    }

    fn spawn_event_loop(
        display_config: DisplayConfig,
        sender: Sender<Event<()>>,
        window_return: Arc<Mutex<Option<Window>>>,
        done: Arc<AtomicBool>,
    ) {
        thread::spawn(move || {
            let event_loop = EventLoop::new();
            let window = display_config
                .into_window_builder(&event_loop)
                .build(&event_loop)
                .unwrap();

            *window_return.lock().unwrap() = Some(window);
            done.store(true, Ordering::SeqCst);

            event_loop.run(move |event, _, &mut _control_flow| {
                sender.send(event).unwrap();
            });
        });
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WindowBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let (receiver, window) = self.build_event_loop_and_window();
        builder.add(WindowSystem::new(world, window), "window", &[]);
        builder.add(EventLoopSystem::new(receiver), "event_loop", &[]);
        Ok(())
    }
}
