//! Utilities for game state management.

use ecs::prelude::World;
use renderer::Event;

/// State data encapsulates the data sent to all state functions from the application main loop.
pub struct StateData<'a, T>
where
    T: 'a,
{
    /// Main `World`
    pub world: &'a mut World,
    /// User defined game data
    pub data: &'a mut T,
}

impl<'a, T> StateData<'a, T>
where
    T: 'a,
{
    /// Create a new state data
    pub fn new(world: &'a mut World, data: &'a mut T) -> Self {
        StateData { world, data }
    }
}

/// Types of state transitions.
pub enum Trans<T> {
    /// Continue as normal.
    None,
    /// Remove the active state and resume the next state on the stack or stop
    /// if there are none.
    Pop,
    /// Pause the active state and push a new state onto the stack.
    Push(Box<State<T>>),
    /// Remove the current state on the stack and insert a different one.
    Switch(Box<State<T>>),
    /// Stop and remove all states and shut down the engine.
    Quit,
}

/// A trait which defines game states that can be used by the state machine.
pub trait State<T> {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<T>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<T>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<T>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<T>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, _data: StateData<T>, _event: Event) -> Trans<T> {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _data: StateData<T>) -> Trans<T> {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow.
    fn update(&mut self, _data: StateData<T>) -> Trans<T> {
        Trans::None
    }
}

/// A simple stack-based state machine (pushdown automaton).
#[derive(Derivative)]
#[derivative(Debug)]
pub struct StateMachine<'a, T> {
    running: bool,
    #[derivative(Debug = "ignore")]
    state_stack: Vec<Box<State<T> + 'a>>,
}

impl<'a, T> StateMachine<'a, T> {
    /// Creates a new state machine with the given initial state.
    pub fn new<S: State<T> + 'a>(initial_state: S) -> StateMachine<'a, T> {
        StateMachine {
            running: false,
            state_stack: vec![Box::new(initial_state)],
        }
    }

    /// Checks whether the state machine is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Initializes the state machine.
    ///
    /// # Panics
    /// Panics if no states are present in the stack.
    pub fn start(&mut self, data: StateData<T>) {
        if !self.running {
            let state = self.state_stack.last_mut().unwrap();
            state.on_start(data);
            self.running = true;
        }
    }

    /// Passes a single event to the active state to handle.
    pub fn handle_event(&mut self, data: StateData<T>, event: Event) {
        let StateData { world, data } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.handle_event(StateData { world, data }, event),
                None => Trans::None,
            };

            self.transition(trans, StateData { world, data });
        }
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, data: StateData<T>) {
        let StateData { world, data } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.fixed_update(StateData { world, data }),
                None => Trans::None,
            };

            self.transition(trans, StateData { world, data });
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, data: StateData<T>) {
        let StateData { world, data } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.update(StateData { world, data }),
                None => Trans::None,
            };

            self.transition(trans, StateData { world, data });
        }
    }

    /// Performs a state transition, if requested by either update() or
    /// fixed_update().
    fn transition(&mut self, request: Trans<T>, data: StateData<T>) {
        if self.running {
            match request {
                Trans::None => (),
                Trans::Pop => self.pop(data),
                Trans::Push(state) => self.push(state, data),
                Trans::Switch(state) => self.switch(state, data),
                Trans::Quit => self.stop(data),
            }
        }
    }

    /// Removes the current state on the stack and inserts a different one.
    fn switch(&mut self, state: Box<State<T>>, data: StateData<T>) {
        if self.running {
            let StateData { world, data } = data;
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData { world, data });
            }

            self.state_stack.push(state);
            let state = self.state_stack.last_mut().unwrap();
            state.on_start(StateData { world, data });
        }
    }

    /// Pauses the active state and pushes a new state onto the state stack.
    fn push(&mut self, state: Box<State<T>>, data: StateData<T>) {
        if self.running {
            let StateData { world, data } = data;
            if let Some(state) = self.state_stack.last_mut() {
                state.on_pause(StateData { world, data });
            }

            self.state_stack.push(state);
            let state = self.state_stack.last_mut().unwrap();
            state.on_start(StateData { world, data });
        }
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    fn pop(&mut self, data: StateData<T>) {
        if self.running {
            let StateData { world, data } = data;
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData { world, data });
            }

            if let Some(state) = self.state_stack.last_mut() {
                state.on_resume(StateData { world, data });
            } else {
                self.running = false;
            }
        }
    }

    /// Shuts the state machine down.
    pub(crate) fn stop(&mut self, data: StateData<T>) {
        if self.running {
            let StateData { world, data } = data;
            while let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData { world, data });
            }

            self.running = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct State1(u8);
    struct State2;

    impl State<()> for State1 {
        fn update(&mut self, _: StateData<()>) -> Trans<()> {
            if self.0 > 0 {
                self.0 -= 1;
                Trans::None
            } else {
                Trans::Switch(Box::new(State2))
            }
        }
    }

    impl State<()> for State2 {
        fn update(&mut self, _: StateData<()>) -> Trans<()> {
            Trans::Pop
        }
    }

    #[test]
    fn switch_pop() {
        use ecs::prelude::World;

        let mut world = World::new();

        let mut sm = StateMachine::new(State1(7));
        sm.start(StateData::new(&mut world, &mut ()));

        for _ in 0..8 {
            sm.update(StateData::new(&mut world, &mut ()));
            assert!(sm.is_running());
        }

        sm.update(StateData::new(&mut world, &mut ()));
        assert!(!sm.is_running());
    }
}
