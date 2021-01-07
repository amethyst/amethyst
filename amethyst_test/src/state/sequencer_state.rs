use amethyst::prelude::*;
use derivative::Derivative;
use derive_new::new;

/// `Push`es each `State` onto the Amethyst state stack in reverse order (LIFO).
///
/// This implementation does not override the `Trans`ition returned by the `State` that is pushed
/// to. Furthermore, it always transitions to the next state in the stack, which means there is no
/// "opt-out" of going through the stack.
#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct SequencerState<T, E>
where
    E: Send + Sync + 'static,
{
    /// States to switch through, in reverse order.
    #[derivative(Debug = "ignore")]
    states: Vec<Box<dyn State<T, E>>>,
}

impl<T, E> State<T, E> for SequencerState<T, E>
where
    E: Send + Sync + 'static,
{
    fn update(&mut self, _data: StateData<'_, T>) -> Trans<T, E> {
        if let Some(state) = self.states.pop() {
            Trans::Push(state)
        } else {
            Trans::Pop
        }
    }
}
