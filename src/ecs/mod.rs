//! `amethyst` engine built-in types for `specs`.

pub use specs::*;

pub mod input;
pub mod transform;
pub mod rendering;
pub mod audio;

use app::ApplicationBuilder;
use error::Result;

/// Describes a bundle of ECS components, resources and systems
pub trait ECSBundle<'a, 'b, T> {
    /// Build and add ECS resources to the world, register components in the world,
    /// and create systems and register them in the dispatcher builder.
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>>;
}
