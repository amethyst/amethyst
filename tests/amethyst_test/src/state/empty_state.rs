use amethyst::prelude::*;

/// Empty Amethyst State that returns `Trans::Pop` on `.update()`.
#[derive(Debug)]
pub struct EmptyState;

impl<T, E> State<T, E> for EmptyState
where
    E: Send + Sync + 'static,
{
    fn update(&mut self, _data: StateData<T>) -> Trans<T, E> {
        Trans::Pop
    }
}
