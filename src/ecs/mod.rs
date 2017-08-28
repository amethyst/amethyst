//! `amethyst` engine built-in types for `specs`.

pub use specs::*;

pub mod input;
pub mod transform;
pub mod rendering;
pub mod audio;

// use config::Config;

use error::Result;

/// Extension trait for all systems.
pub trait SystemExt<'a, A>: System<'a> {
    /// Constructs a new system with the given arguments.
    /// Registers all supported components with the World.
    /// Puts resources into World.
    fn build(args: A, world: &mut World) -> Result<Self>
    where
        Self: Sized;
}
