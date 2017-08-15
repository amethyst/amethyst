//! Built-in `specs` `System`s.

pub use self::rendering::RenderSystem;
pub use self::transform::TransformSystem;

// use config::Config;
use error::Result;
use ecs::{System, World};

mod rendering;
mod transform;

/// Extension trait for all systems.
pub trait SystemExt<'a, A>: System<'a> {
    /// Constructs a new system with the given arguments. 
    /// Registers all supported components with the World.
    /// Puts resources into World.
    fn build(args: A, world: &mut World) -> Result<Self> where Self: Sized;
}