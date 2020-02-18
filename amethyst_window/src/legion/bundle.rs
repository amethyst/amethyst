use crate::{config::DisplayConfig, legion::*};
use amethyst_config::Config;
use amethyst_core::{
    legion::{
        dispatcher::{Stage, SystemBundle},
        prelude::*,
    },
    shrev::EventChannel,
};
use amethyst_error::Error;
use winit::{Event, EventsLoop};

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
        WindowBundle::from_config(DisplayConfig::load(path.as_ref()).unwrap())
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
}

impl SystemBundle for WindowBundle {
    fn build(
        self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), amethyst_error::Error> {
        let event_loop = EventsLoop::new();

        let window = self
            .config
            .into_window_builder(&event_loop)
            .build(&event_loop)
            .unwrap();

        builder.add_system(Stage::Render, move |world, resources| {
            window_system::build(world, resources, window)
        });
        builder.add_thread_local_system(Stage::Begin, move |world, resources| {
            events_loop_system::build(world, resources, event_loop)
        });

        Ok(())
    }
}
