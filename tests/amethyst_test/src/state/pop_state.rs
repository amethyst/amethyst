use amethyst::prelude::*;

/// State that returns `Trans::Pop` on `.update()`.
#[derive(Debug)]
pub struct PopState;

impl<T, E> State<T, E> for PopState
where
    E: Send + Sync + 'static,
{
    fn update(&mut self, _data: StateData<'_, T>) -> Trans<T, E> {
        Trans::Pop
    }
}
