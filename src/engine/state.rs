//! Utilities for game state management.

use engine::Engine;
use engine::event::WindowEvent;

/// Types of state transitions.
pub enum Trans {
    /// Continue as normal.
    None,
    /// Remove the active state and resume the next state on the stack or stop
    /// if there are none.
    Pop,
    /// Pause the active state and push a new state onto the stack.
    Push(Box<State>),
    /// Remove the current state on the stack and insert a different one.
    Switch(Box<State>),
    /// Stop and remove all states and shut down the engine.
    Quit,
}

/// A trait which defines game states that can be used by the state machine.
pub trait State {
    /// Executed when the game state begins.
    #[allow(unused)]
    fn on_start(&mut self, engine: &mut Engine) {}

    /// Executed when the game state exits.
    #[allow(unused)]
    fn on_stop(&mut self, engine: &mut Engine) {}

    /// Executed when a different game state is pushed onto the stack.
    #[allow(unused)]
    fn on_pause(&mut self, engine: &mut Engine) {}

    /// Executed when the application returns to this game state once again.
    #[allow(unused)]
    fn on_resume(&mut self, engine: &mut Engine) {}

    /// Executed on every frame before updating, for use in reacting to events.
    #[allow(unused)]
    fn handle_events(&mut self, events: &[WindowEvent], engine: &mut Engine) -> Trans {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    #[allow(unused)]
    fn fixed_update(&mut self, engine: &mut Engine) -> Trans {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow.
    #[allow(unused)]
    fn update(&mut self, engine: &mut Engine) -> Trans {
        Trans::None
    }
}

/// A simple stack-based state machine (pushdown automaton).
pub struct StateMachine {
    running: bool,
    state_stack: Vec<Box<State>>,
}

impl StateMachine {
    /// Creates a new state machine with the given initial state.
    pub fn new<T>(initial_state: T) -> StateMachine
        where T: State + 'static
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
    ///
    /// # Panics
    /// Panics if no states are present in the stack.
    pub fn start(&mut self, engine: &mut Engine) {
        if !self.running {
            let state = self.state_stack.last_mut().unwrap();
            state.on_start(engine);
            self.running = true;
        }
    }

    /// Passes a vector of events to the active state to handle.
    pub fn handle_events(&mut self, events: &[WindowEvent], engine: &mut Engine) {

        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.handle_events(events, engine),
                None => Trans::None,
            };

            self.transition(trans, engine);
        }
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, engine: &mut Engine) {
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.fixed_update(engine),
                None => Trans::None,
            };

            self.transition(trans, engine);
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, engine: &mut Engine) {
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.update(engine),
                None => Trans::None,
            };

            self.transition(trans, engine);
        }
    }

    /// Performs a state transition, if requested by either update() or
    /// fixed_update().
    fn transition(&mut self, request: Trans, engine: &mut Engine) {
        if self.running {
            match request {
                Trans::None => (),
                Trans::Pop => self.pop(engine),
                Trans::Push(state) => self.push(state, engine),
                Trans::Switch(state) => self.switch(state, engine),
                Trans::Quit => self.stop(engine),
            }
        }
    }

    /// Removes the current state on the stack and inserts a different one.
    fn switch(&mut self, state: Box<State>, engine: &mut Engine) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(engine);
            }

            self.state_stack.push(state);
            let state = self.state_stack.last_mut().unwrap();
            state.on_start(engine);
        }
    }

    /// Pauses the active state and pushes a new state onto the state stack.
    fn push(&mut self, state: Box<State>, engine: &mut Engine) {
        if self.running {
            if let Some(state) = self.state_stack.last_mut() {
                state.on_pause(engine);
            }

            self.state_stack.push(state);
            let state = self.state_stack.last_mut().unwrap();
            state.on_start(engine);
        }
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    fn pop(&mut self, engine: &mut Engine) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(engine);
            }

            if let Some(mut state) = self.state_stack.last_mut() {
                state.on_resume(engine);
            } else {
                self.running = false;
            }
        }
    }

    /// Shuts the state machine down.
    fn stop(&mut self, engine: &mut Engine) {
        if self.running {
            while let Some(mut state) = self.state_stack.pop() {
                state.on_stop(engine);
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

    impl State for State1 {
        fn update(&mut self, _: &mut World, _: &mut AssetLoader, _: &mut Pipeline) -> Trans {
            if self.0 > 0 {
                self.0 -= 1;
                Trans::None
            } else {
                Trans::Switch(Box::new(State2))
            }
        }
    }

    impl State for State2 {
        fn update(&mut self, _: &mut World, _: &mut AssetLoader, _: &mut Pipeline) -> Trans {
            Trans::Pop
        }
    }

    #[test]
    fn switch_pop() {
        let mut asset_loader = AssetLoader::new();
        let mut pipe = Pipeline::new();
        let mut world = World::new();

        let mut sm = StateMachine::new(State1(7));
        sm.start(&mut world, &mut asset_loader, &mut pipe);

        for _ in 0..8 {
            sm.update(&mut world, &mut asset_loader, &mut pipe);
            assert!(sm.is_running());
        }

        sm.update(&mut world, &mut asset_loader, &mut pipe);
        assert!(!sm.is_running());
    }
}
