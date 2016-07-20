//! Utilities for game state management.

use context::Context;
use ecs::Entity;
use super::Duration;

/// Types of state transitions.
pub enum Trans {
    /// Continue as normal.
    None,
    /// Remove the active state and resume the next state on the stack or stop if there are none.
    Pop,
    /// Pause the active state and push a new state onto the stack.
    Push(Box<State>),
    /// Remove the current state on the stack and insert a different one.
    Switch(Box<State>),
    /// Stop and remove all states and shut down the engine.
    Quit,
}

#[allow(unused_variables)]
/// A trait which defines game states that can be used by the state machine.
pub trait State {
    /// Executed when the game state begins.
    fn on_start(&mut self, context: &mut Context) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, context: &mut Context) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, context: &mut Context) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, context: &mut Context) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_events(&mut self, _events: Vec<Entity>, context: &mut Context) -> Trans { Trans::None }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _delta: Duration, context: &mut Context) -> Trans { Trans::None }

    /// Executed on every frame immediately, as fast as the engine will allow.
    fn update(&mut self, _delta: Duration, context: &mut Context) -> Trans { Trans::Pop }
}
#[warn(unused_variables)]

/// A simple stack-based state machine (pushdown automaton).
pub struct StateMachine {
    running: bool,
    state_stack: Vec<Box<State>>,
}

impl StateMachine {
    pub fn new<T: 'static>(initial_state: T) -> StateMachine
        where T: State
    {
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
    /// # Panics
    ///	Panics if no states are present in the stack.
    pub fn start(&mut self, context: &mut Context) {
        if !self.running {
            self.state_stack.last_mut().unwrap().on_start(context);
            self.running = true;
        }
    }

    /// Passes a vector of events to the active state to handle.
    // TODO: Replace i32 with an actual Event type of some kind.
    pub fn handle_events(&mut self, events: Vec<Entity>, context: &mut Context) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(state) = self.state_stack.last_mut() {
                trans = state.handle_events(events, context);
            }
            self.transition(trans, context);
        }
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, delta_time: Duration, context: &mut Context) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(state) = self.state_stack.last_mut() {
                trans = state.fixed_update(delta_time, context);
            }
            self.transition(trans, context);
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, delta_time: Duration, context: &mut Context) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(state) = self.state_stack.last_mut() {
                trans = state.update(delta_time, context);
            }
            self.transition(trans, context);
        }
    }

    /// Performs a state transition, if requested by either update() or
    /// fixed_update().
    fn transition(&mut self, request: Trans, context: &mut Context) {
        if self.running {
            match request {
                Trans::None => (),
                Trans::Pop => self.pop(context),
                Trans::Push(state) => self.push(state, context),
                Trans::Switch(state) => self.switch(state, context),
                Trans::Quit => self.stop(context),
            }
        }
    }

    /// Removes the current state on the stack and inserts a different one.
    fn switch(&mut self, state: Box<State>, context: &mut Context) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(context);
            }

            self.state_stack.push(state);
            self.state_stack.last_mut().unwrap().on_start(context);
        }
    }

    /// Pauses the active state and pushes a new state onto the state stack.
    fn push(&mut self, state: Box<State>, context: &mut Context) {
        if self.running {
            if let Some(state) = self.state_stack.last_mut() {
                state.on_pause(context);
            }

            self.state_stack.push(state);
            self.state_stack.last_mut().unwrap().on_start(context);
        }
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    fn pop(&mut self, context: &mut Context) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(context);
            }

            if let Some(state) = self.state_stack.last_mut() {
                state.on_resume(context);
            } else {
                self.running = false;
            }
        }
    }

    /// Shuts the state machine down.
    fn stop(&mut self, context: &mut Context) {
        if self.running {
            while let Some(mut state) = self.state_stack.pop() {
                state.on_stop(context);
            }

            self.running = false;
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use engine::Duration;

    struct State1(u8);
    struct State2;

    impl State for State1 {
        fn update(&mut self, _delta: Duration) -> Trans {
            if self.0 > 0 {
                self.0 -= 1;
                Trans::None
            } else {
                Trans::Switch(Box::new(State2))
            }
        }
    }

    impl State for State2 {
        fn update(&mut self, _delta: Duration) -> Trans {
            Trans::Pop
        }
    }

    #[test]
    fn switch_pop() {
        let mut sm = StateMachine::new(State1(7));
        sm.start();
        for _ in 0..8 {
            sm.update(Duration::seconds(0));
            assert!(sm.is_running());
        }
        sm.update(Duration::seconds(0));
        assert!(!sm.is_running());
    }
}
