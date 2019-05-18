use crate::{DisplayConfig, EventsLoopSystem, WindowSystem};
use amethyst_config::Config;
use amethyst_core::{bundle::SystemBundle, shred::DispatcherBuilder};
use amethyst_error::Error;
use winit::EventsLoop;

pub struct WindowBundle {
    config: DisplayConfig,
}

impl WindowBundle {
    /// Builds a new window bundle from a loaded `DisplayConfig`.
    pub fn from_config(config: DisplayConfig) -> Self {
        WindowBundle { config }
    }

    /// Builds a new window bundle by loading the `DisplayConfig` from `path`.
    /// Will fall back to `DisplayConfig::default()` in case of an error.
    pub fn from_config_path(path: impl AsRef<std::path::Path>) -> Self {
        WindowBundle::from_config(DisplayConfig::load(path.as_ref()))
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WindowBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        let event_loop = EventsLoop::new();
        builder.add(
            WindowSystem::from_config(&event_loop, self.config),
            "window",
            &[],
        );
        builder.add_thread_local(EventsLoopSystem::new(event_loop));
        Ok(())
    }
}
