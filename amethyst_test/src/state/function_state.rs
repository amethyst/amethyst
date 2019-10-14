use amethyst::prelude::*;

use crate::GameUpdate;

/// Runs a function in `.update()` then `Pop`s itself.
///
/// The function is run before `GameUpdate#update(world)` is called.
#[derive(Debug)]
pub struct FunctionState<F>
where
    F: FnOnce(&mut World),
{
    /// Function to run in `update`.
    function: Option<F>,
}

impl<F> FunctionState<F>
where
    F: FnOnce(&mut World),
{
    /// Returns a new `FunctionState`
    pub fn new(function: F) -> Self {
        FunctionState {
            function: Some(function),
        }
    }
}

impl<F, T, E> State<T, E> for FunctionState<F>
where
    F: FnOnce(&mut World),
    T: GameUpdate,
    E: Send + Sync + 'static,
{
    fn update(&mut self, mut data: StateData<'_, T>) -> Trans<T, E> {
        data.data.update(&data.world);

        if let Some(function) = self.function.take() {
            (function)(&mut data.world);
        }

        Trans::Pop
    }
}
