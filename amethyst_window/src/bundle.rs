use amethyst_config::{Config, ConfigError};
use amethyst_core::ecs::*;
use amethyst_error::Error;
use winit::event_loop::EventLoop;

use crate::{DisplayConfig, EventLoopSystem, ScreenDimensions, WindowSystem};

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
    pub fn from_config_path(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        Ok(WindowBundle::from_config(DisplayConfig::load(
            path.as_ref(),
        )?))
    }

    /// Builds a new window bundle with a predefined `DisplayConfig`.
    ///
    /// This uses a `DisplayConfig::default()`, but with the following differences:
    ///
    /// * `dimensions` is changed to `Some((SCREEN_WIDTH, SCREEN_HEIGHT))`.
    /// * `visibility` is `false`.
    #[cfg(feature = "test-support")]
    pub fn from_test_config() -> Self {
        let display_config = DisplayConfig {
            dimensions: Some((SCREEN_WIDTH, SCREEN_HEIGHT)),
            visibility: false,
            ..Default::default()
        };

        WindowBundle::from_config(display_config)
    }
}

impl SystemBundle for WindowBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        let event_loop: EventLoop<()> = EventLoop::new();

        let window = self
            .config
            .clone()
            .into_window_builder(&event_loop)
            .build(&event_loop)
            .expect("Unable to create window");

        let (width, height) = window.inner_size().into();

        resources.insert(ScreenDimensions::new(width, height));
        resources.insert(window);

        builder
            .add_system(WindowSystem)
            .add_thread_local(EventLoopSystem { event_loop });

        Ok(())
    }
}
