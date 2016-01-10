use super::state::State;

enum Transition {
    Nothing,
    Pop,
    Push,
    Quit,
    Switch,
}

/// A handle to the game state machine.
pub struct Actions {
    next_state: Option<Box<State>>,
    transition: Transition,
}

impl Actions {
    pub fn new() -> Actions {
        Actions {
            next_state: None
            transition: Transition::Nothing,
        }
    }

    /// Signals to the engine to push a state onto the stack.
    pub fn push<T: 'static>(&mut self, state: T)
        where T: State
    {
        self.transition = Transition::Push;
    }

    /// Signals to the engine to switch states.
    pub fn switch<T: 'static>(&mut self, state: T)
        where T: State
    {
        self.transition = Transition::Switch;
    }

    /// Signals to the engine to quit.
    pub fn quit(&mut self) {
        self.transition = Transition::Quit;
    }

    pub fn get_trans(&mut self) -> (Transition, Option<Box<State>>) {
        (self.transition, self.next_state)
    }
}
