use std::fmt::Debug;

use amethyst_core::ecs::{System, World};

/// Initializes a `System` with some interaction with the `World`.
pub trait SystemDesc<'a, 'b>: Debug {
    /// Builds and returns a `System`.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` that the system will run on.
    fn build<S>(self, world: &mut World) -> S
    where
        S: System<'a>;
}
