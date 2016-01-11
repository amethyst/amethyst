//! Utilities for game state management.

use super::timing::Duration;

/// Types of state transitions.
pub enum Trans {
    None,
    Pop,
    Push(Box<State>),
    Switch(Box<State>),
    Quit,
}

/// A trait which defines game states that can be used by the state machine.
pub trait State {
    /// Executed when the game state begins.
    fn on_start(&mut self) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self) {}

    /// Executed on every frame before updating, for use in reacting to events.
    // TODO: Replace i32 with an actual Event type of some kind.
    fn handle_events(&mut self, _events: &Vec<i32>) {}

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _delta: Duration) -> Trans { Trans::None }

    /// Executed on every frame immediately, as fast as the engine will allow.
    fn update(&mut self, _delta: Duration) -> Trans { Trans::Pop }
}

/// A simple stack-based state machine.
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
    pub fn start(&mut self) {
        if !self.running {
            self.state_stack.last_mut().unwrap().on_start();
            self.running = true;
        }
    }

    /// Passes a vector of events to the active state to handle.
    // TODO: Replace i32 with an actual Event type of some kind.
    pub fn handle_events(&mut self, events: &Vec<i32>) {
        if self.running {
            if let Some(state) = self.state_stack.last_mut() {
                state.handle_events(events);
            }
        }
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, delta_time: Duration) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(state) = self.state_stack.last_mut() {
                trans = state.fixed_update(delta_time);
            }
            self.transition(trans);
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, delta_time: Duration) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(state) = self.state_stack.last_mut() {
                trans = state.update(delta_time);
            }
            self.transition(trans);
        }
    }

    /// Performs a state transition, if requested by either update() or
    /// fixed_update().
    fn transition(&mut self, request: Trans) {
        if self.running {
            match request {
                Trans::None => (),
                Trans::Pop => self.pop(),
                Trans::Push(state) => self.push(state),
                Trans::Switch(state) => self.switch(state),
                Trans::Quit => self.stop(),
            }
        }
    }

    /// Removes the current state on the stack and inserts a different one.
    fn switch(&mut self, state: Box<State>) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop();
            }

            self.state_stack.push(state);
            self.state_stack.last_mut().unwrap().on_start();
        }
    }

    /// Pauses the active state (if any) and pushes a new state onto the state
    /// stack.
    fn push(&mut self, state: Box<State>) {
        if self.running {
            if let Some(state) = self.state_stack.last_mut() {
                state.on_pause();
            }

            self.state_stack.push(state);
            self.state_stack.last_mut().unwrap().on_start();
        }
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    fn pop(&mut self) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop();
            }

            if let Some(state) = self.state_stack.last_mut() {
                state.on_resume();
            } else {
                self.running = false;
            }
        }
    }

    /// Shuts the state machine down.
    fn stop(&mut self) {
        if self.running {
            while let Some(mut state) = self.state_stack.pop() {
                state.on_stop();
            }

            self.running = false;
        }
    }
}

