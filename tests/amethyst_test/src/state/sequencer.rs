use amethyst::prelude::*;
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Step<S> {
    /// Don't do anything.
    None,
    /// Continue to the next step.
    Next,
    /// Apply the given transition and continue to the next step.
    Trans(Trans<S>),
}

impl<S> From<()> for Step<S> {
    fn from(_: ()) -> Step<S> {
        Step::Next
    }
}

impl<S> From<Trans<S>> for Step<S> {
    fn from(trans: Trans<S>) -> Step<S> {
        Step::Trans(trans)
    }
}

/// A function that operates on the world and returns a transition.
pub type WorldFn<S> = Box<for<'w> FnMut(&'w mut World) -> Step<S> + Send + Sync>;

/// Forwards any GlobalCallback's one at a time to a list of nested callbacks.
#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct Sequencer<S> {
    /// States to switch through, in reverse order.
    #[derivative(Debug = "ignore")]
    functions: VecDeque<WorldFn<S>>,
}

impl<S, E> GlobalCallback<S, E> for Sequencer<S>
where
    S: 'static + Send + Sync + State<E>,
    E: 'static + Send + Sync,
{
    fn update(&mut self, world: &mut World) -> Trans<S> {
        if self.functions.is_empty() {
            return Trans::Quit;
        }

        let mut trans = Trans::None;

        let pop = if let Some(func) = self.functions.front_mut() {
            match func(world) {
                Step::None => false,
                Step::Next => true,
                Step::Trans(t) => {
                    trans = t;
                    true
                }
            }
        } else {
            false
        };

        if pop {
            self.functions.pop_front();
        }

        trans
    }
}
