//! Utilities for game state management.

use super::timing::Duration;

/// A trait which defines game states that can be used by the state machine.
pub trait State {
    /// Standard constructor for all game states.
    fn new() -> Self where Self: Sized;

    /// Executed the first time when the game state is reached.
    fn on_start(&mut self) {}

    /// Executed when the application finally exits.
    fn on_stop(&mut self) {}

    /// Executed when the application switches away to a different game state.
    fn on_pause(&mut self) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self) {}

    /// Executed on every frame before updating, for use in reacting to events.
    /// TODO: Replace i32 with an actual Event type of some kind.
    fn handle_events(&mut self, _events: &Vec<i32>) {}

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _delta: Duration, _game: &mut Actions) {}

    /// Executed on every frame immediately, as fast as the engine will allow.
    fn update(&mut self, _delta: Duration, _game: &mut Actions) {}
}

/// A possible action to take.
#[derive(Clone, Copy)]
pub enum Transition {
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
            next_state: None,
            transition: Transition::Nothing,
        }
    }

    /// Signals to the engine to push a state onto the stack.
    pub fn push<T: 'static>(&mut self, state: T)
        where T: State
    {
        self.transition = Transition::Push;
        self.next_state = Some(Box::new(state));
    }

    /// Signals to the engine to switch states.
    pub fn switch<T: 'static>(&mut self, state: T)
        where T: State
    {
        self.transition = Transition::Switch;
        self.next_state = Some(Box::new(state));
    }

    /// Signals to the engine to quit.
    pub fn quit(&mut self) {
        self.transition = Transition::Quit;
    }

    pub fn get_trans(&mut self) -> (Transition, Option<Box<State>>) {
        let output = (self.transition.clone(), self.next_state.take());
        self.transition = Transition::Nothing;
        output
    }
}

/// A simple stack-based state machine.
pub struct StateMachine {
    actions: Actions,
    running: bool,
    state_stack: Vec<Box<State>>,
}

impl StateMachine {
    pub fn new<T: 'static>(initial_state: T) -> StateMachine
        where T: State
    {
        StateMachine {
            actions: Actions::new(),
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
            if let Some(state) = self.state_stack.last_mut() {
                state.fixed_update(delta_time, &mut self.actions);
            }
            self.transition();
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, delta_time: Duration) {
        if self.running {
            if let Some(state) = self.state_stack.last_mut() {
                state.update(delta_time, &mut self.actions);
            }
            self.transition();
        }
    }

    /// Makes a state transition if flagged by the Actions struct.
    fn transition(&mut self) {
        let trans = self.actions.get_trans();
        match trans.0 {
            Transition::Nothing => (),
            Transition::Pop => self.pop(),
            Transition::Push => self.push(trans.1.unwrap()),
            Transition::Quit => self.stop(),
            Transition::Switch => self.switch(trans.1.unwrap()),
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
