use amethyst::prelude::*;

use derive_new::new;

use crate::GameUpdate;

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

impl<F, T, E> State<T, E> for FunctionState<F>
where
    F: Fn(&mut World),
    T: GameUpdate,
    E: Send + Sync + 'static,
{
    fn update(&mut self, mut data: StateData<'_, T>) -> Trans<T, E> {
        data.data.update(&data.world);

        (self.function)(&mut data.world);

        Trans::Pop
    }
}
