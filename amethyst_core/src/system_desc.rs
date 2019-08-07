use specs::{System, World};

/// Initializes a `System` with some interaction with the `World`.
pub trait SystemDesc<'a, 'b, S>
where
    S: System<'a>,
{
    /// Builds and returns a `System`.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` that the system will run on.
    fn build(self, world: &mut World) -> S;
}
