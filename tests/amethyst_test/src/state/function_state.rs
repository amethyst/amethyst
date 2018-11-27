use amethyst::prelude::*;

/// Runs a function in `.update()` then `Pop`s itself.
///
/// The function is run before `GameUpdate#update(world)` is called.
#[derive(Debug, new)]
pub struct FunctionState<F>
where
    F: Fn(&mut World),
{
    /// Function to run in `update`.
    function: F,
}

impl<F, S, E> GlobalCallback<S, E> for FunctionState<F>
where
    F: Fn(&mut World),
    E: Send + Sync + 'static,
{
    fn update(&mut self, world: &mut World) -> Trans<S> {
        (self.function)(world);
        Trans::Pop
    }
}

impl<F, S, E> StateCallback<S, E> for FunctionState<F>
where
    F: Fn(&mut World),
    E: Send + Sync + 'static,
{
    fn update(&mut self, world: &mut World) -> Trans<S> {
        (self.function)(world);
        Trans::Pop
    }
}
