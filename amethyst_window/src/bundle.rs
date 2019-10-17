use crate::{DisplayConfig, EventsLoopSystem, WindowSystem};
use amethyst_config::{Config, ConfigError};
use amethyst_core::{bundle::SystemBundle, ecs::World, shred::DispatcherBuilder};
use amethyst_error::Error;
use winit::EventsLoop;

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
        let mut display_config = DisplayConfig::default();
        display_config.dimensions = Some((SCREEN_WIDTH, SCREEN_HEIGHT));
        display_config.visibility = false;

        WindowBundle::from_config(display_config)
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WindowBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        let event_loop = EventsLoop::new();
        builder.add(
            WindowSystem::from_config(world, &event_loop, self.config),
            "window",
            &[],
        );
        builder.add_thread_local(EventsLoopSystem::new(event_loop));
        Ok(())
    }
}
