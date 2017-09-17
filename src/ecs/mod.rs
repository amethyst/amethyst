//! `amethyst` engine built-in types for `specs`.

pub use specs::*;

pub mod input;
pub mod transform;
pub mod rendering;
pub mod audio;

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

/// Describes a bundle of ECS components, resources and systems
pub trait ECSBundle<'a, 'b, A> {
    /// Build and add ECS resources to the world, register components in the world,
    /// and create systems and register them in the dispatcher builder.
    fn build(
        &self,
        args: A,
        world: &mut World,
        dispatcher: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>>;
}
