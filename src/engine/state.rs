//! Utilities for game state management.

use context::Context;
use ecs::{Entity, Planner, World};
use std::sync::{Arc, Mutex};

/// Types of state transitions.
pub enum Trans {
    /// Continue as normal.
    None,
    /// Remove the active state and resume the next state on the stack or stop if there are none.
    Pop,
    /// Pause the active state and push a new state onto the stack.
    Push(Box<State>, Planner<Arc<Mutex<Context>>>),
    /// Remove the current state on the stack and insert a different one.
    Switch(Box<State>, Planner<Arc<Mutex<Context>>>),
    /// Stop and remove all states and shut down the engine.
    Quit,
}

#[allow(unused_variables)]
/// A trait which defines game states that can be used by the state machine.
pub trait State {
    /// Executed when the game state begins.
    fn on_start(&mut self, context: &mut Context, world: &mut World) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, context: &mut Context, world: &mut World) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, context: &mut Context, world: &mut World) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, context: &mut Context, world: &mut World) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_events(&mut self, _events: Vec<Entity>, context: &mut Context, world: &mut World) -> Trans { Trans::None }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, context: &mut Context, world: &mut World) -> Trans { Trans::None }

    /// Executed on every frame immediately, as fast as the engine will allow.
    fn update(&mut self, context: &mut Context, world: &mut World) -> Trans { Trans::Pop }
}
#[warn(unused_variables)]

/// A simple stack-based state machine (pushdown automaton).
pub struct StateMachine {
    running: bool,
    state_stack: Vec<(Box<State>, Planner<Arc<Mutex<Context>>>)>,
}

impl StateMachine {
    pub fn new<T: 'static>(initial_state: T, planner: Planner<Arc<Mutex<Context>>>) -> StateMachine
        where T: State
    {
        StateMachine {
            running: false,
            state_stack: vec![(Box::new(initial_state), planner)],
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
            let (ref mut state, ref mut planner) = *self.state_stack.last_mut().unwrap();
            state.on_start(context, planner.mut_world());
            self.running = true;
        }
    }

    /// Runs processors if the state machine is running.
    pub fn run_processors(&mut self, context: Arc<Mutex<Context>>) {
        if self.running {
            let (_, ref mut planner) = *self.state_stack.last_mut().unwrap();
            planner.dispatch(context);
        }
    }

    /// Passes a vector of events to the active state to handle.
    // TODO: Replace i32 with an actual Event type of some kind.
    pub fn handle_events(&mut self, events: Vec<Entity>, context: &mut Context) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(_state) = self.state_stack.last_mut() {
                let (ref mut state, ref mut planner) = *_state;
                trans = state.handle_events(events, context, planner.mut_world());
            }
            self.transition(trans, context);
        }
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, context: &mut Context) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(_state) = self.state_stack.last_mut() {
                let (ref mut state, ref mut planner) = *_state;
                trans = state.fixed_update(context, planner.mut_world());
            }
            self.transition(trans, context);
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, context: &mut Context) {
        if self.running {
            let mut trans = Trans::None;
            if let Some(_state) = self.state_stack.last_mut() {
                let (ref mut state, ref mut planner) = *_state;
                trans = state.update(context, planner.mut_world());
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
                Trans::Push(state, planner) => self.push(state, planner, context),
                Trans::Switch(state, planner) => self.switch(state, planner, context),
                Trans::Quit => self.stop(context),
            }
        }
    }

    /// Removes the current state on the stack and inserts a different one.
    fn switch(&mut self, state: Box<State>, planner: Planner<Arc<Mutex<Context>>>, context: &mut Context) {
        if self.running {
            if let Some(mut _state) = self.state_stack.pop() {
                let (ref mut state, ref mut planner) = _state;
                state.on_stop(context, planner.mut_world());
            }

            self.state_stack.push((state, planner));
            let (ref mut state, ref mut planner) = *self.state_stack.last_mut().unwrap();
            state.on_start(context, planner.mut_world());
        }
    }

    /// Pauses the active state and pushes a new state onto the state stack.
    fn push(&mut self, state: Box<State>, planner: Planner<Arc<Mutex<Context>>>, context: &mut Context) {
        if self.running {
            if let Some(_state) = self.state_stack.last_mut() {
                let (ref mut state, ref mut planner) = *_state;
                state.on_pause(context, planner.mut_world());
            }

            self.state_stack.push((state, planner));
            let (ref mut state, ref mut planner) = *self.state_stack.last_mut().unwrap();
            state.on_start(context, planner.mut_world());
        }
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    fn pop(&mut self, context: &mut Context) {
        if self.running {
            if let Some(mut _state) = self.state_stack.pop() {
                let (ref mut state, ref mut planner) = _state;
                state.on_stop(context, planner.mut_world());
            }

            if let Some(_state) = self.state_stack.last_mut() {
                let (ref mut state, ref mut planner) = *_state;
                state.on_resume(context, planner.mut_world());
            } else {
                self.running = false;
            }
        }
    }

    /// Shuts the state machine down.
    fn stop(&mut self, context: &mut Context) {
        if self.running {
            while let Some(mut _state) = self.state_stack.pop() {
                let (ref mut state, ref mut planner) = _state;
                state.on_stop(context, planner.mut_world());
            }

            self.running = false;
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use context::{Config, Context};
    use ecs::{Planner, World};

    struct State1(u8);
    struct State2;

    impl State for State1 {
        fn update(&mut self, _: &mut Context, _: &mut World) -> Trans {
            if self.0 > 0 {
                self.0 -= 1;
                Trans::None
            } else {
                let world = World::new();
                let planner = Planner::new(world, 1);
                Trans::Switch(Box::new(State2), planner)
            }
        }
    }

    impl State for State2 {
        fn update(&mut self, _: &mut Context, _: &mut World) -> Trans {
            Trans::Pop
        }
    }

    #[test]
    fn switch_pop() {
        let config = Config::default();
        let mut context = Context::new(config);
        let world = World::new();
        let planner = Planner::new(world, 1);
        let mut sm = StateMachine::new(State1(7), planner);
        sm.start(&mut context);
        for _ in 0..8 {
            sm.update(&mut context);
            assert!(sm.is_running());
        }
        sm.update(&mut context);
        assert!(!sm.is_running());
    }
}
