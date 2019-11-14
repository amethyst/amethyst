use crate::{DisplayConfig, WindowSystem, EventLoop};
use amethyst_config::{Config, ConfigError};
use amethyst_core::{bundle::SystemBundle, ecs::World, shred::DispatcherBuilder};
use amethyst_error::Error;
use std::sync::{Arc, Mutex};
use amethyst_core::shred::cell::TrustCell;


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
}

impl WindowBundle {
    pub fn new() -> Self {
        Self { }
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WindowBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        
        builder.add_thread_local(
            WindowSystem::new()
        );
        // world.insert(TrustCell::new(event_loop));
        // builder.add_thread_local(EventsLoopSystem::new(event_loop));
        Ok(())
    }
}
