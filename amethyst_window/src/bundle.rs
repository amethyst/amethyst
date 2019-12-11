use crate::WindowSystem;
use amethyst_core::{bundle::SystemBundle, ecs::World, shred::DispatcherBuilder};
use amethyst_error::Error;

/// Screen width used in predefined display configuration.
#[cfg(feature = "test-support")]
pub const SCREEN_WIDTH: u32 = 800;
/// Screen height used in predefined display configuration.
#[cfg(feature = "test-support")]
pub const SCREEN_HEIGHT: u32 = 600;

/// Bundle providing easy initializing of the `WindowSystem` used for managing WindowDimensions
#[derive(Debug)]
pub struct WindowBundle {}

impl WindowBundle {
    /// Creates new WindowBundle
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for WindowBundle {
    fn build(self, _: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add_thread_local(WindowSystem::new());

        Ok(())
    }
}
