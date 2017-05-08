//! Built-in `specs` `System`s.

pub use self::rendering::RenderingSystem;
pub use self::transform::TransformSystem;

use config::Config;
use error::Result;
use ecs::{System, World};
use engine::EventsIter;

mod rendering;
mod transform;

/// Extension trait for all systems.
pub trait SystemExt: System<()> {
    /// Constructs a new system with the given configuration. 
    fn build(cfg: &Config) -> Result<Self> where Self: Sized;

    /// Registers all supported components with the World.
    fn register(world: &mut World);

    /// Polls the system's event queue.
    fn poll_events(&self) -> EventsIter {
        EventsIter::default()
    }
}
