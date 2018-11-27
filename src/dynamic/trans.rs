//! Type describing state transitions.

/// Types of state transitions.
/// `S` is the state this state machine deals with.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Trans<S> {
    /// Continue as normal.
    None,
    /// Remove the active state and resume the next state on the stack or stop
    /// if there are none.
    Pop,
    /// Pause the active state and push a new state onto the stack.
    Push(S),
    /// Remove the current state on the stack and insert a different one.
    Switch(S),
    /// Stop and remove all states and shut down the engine.
    Quit,
}

impl<S> Trans<S> {
    /// Update this transition with another, unless it is `Trans::None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use amethyst::Trans;
    ///
    /// let mut trans: Trans<()> = Trans::None;
    ///
    /// trans.update(Trans::Pop);
    /// assert_eq!(trans, Trans::Pop);
    ///
    /// trans.update(Trans::None);
    /// assert_eq!(trans, Trans::Pop);
    ///
    /// trans.update(Trans::Quit);
    /// assert_eq!(trans, Trans::Quit);
    /// ```
    pub fn update(&mut self, other: Trans<S>) {
        match other {
            Trans::None => (),
            other => *self = other,
        }
    }
}

/// Transition converted into an iterator.
pub struct Iter<S> {
    next: Trans<S>,
}

impl<S> Iterator for Iter<S> {
    type Item = Trans<S>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::mem;

        // Note: Iterating over `Trans::None` should yield no transitions.
        if let Trans::None = self.next {
            return None;
        }

        Some(mem::replace(&mut self.next, Trans::None))
    }
}

impl<S> IntoIterator for Trans<S> {
    type Item = Trans<S>;
    type IntoIter = Iter<S>;

    fn into_iter(self) -> Iter<S> {
        Iter { next: self }
    }
}

/// Event queue to trigger state `Trans` from other places than a `State`'s methods.
/// # Example:
/// ```rust, ignore
/// world.write_resource::<EventChannel<TransEvent<State>>>().single_write(Box::new(|| Trans::Quit));
/// ```
///
/// Transitions will be executed sequentially by Amethyst's `CoreApplication` update loop.
pub type TransEvent<S> = Box<dyn Fn() -> Trans<S> + Send + Sync + 'static>;

impl<S> From<()> for Trans<S> {
    fn from(_: ()) -> Trans<S> {
        Trans::None
    }
}
