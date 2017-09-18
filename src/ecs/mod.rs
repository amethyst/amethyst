//! `amethyst` engine built-in types for `specs`.

pub use specs::*;

pub mod input;
pub mod transform;
pub mod rendering;
pub mod audio;

use app::ApplicationBuilder;
use error::Result;

/// A bundle of ECS components, resources and systems.
pub trait ECSBundle<'a, 'b, T> {
    /// Build and add ECS resources, register components, add systems etc to the Application.
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>>;
}
