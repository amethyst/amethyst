//! Utilities for game state management.

use amethyst_input::is_close_requested;
use ecs::prelude::World;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use {GameData, StateEvent};

/// Error type for errors occurring in StateMachine
#[derive(Debug)]
pub enum StateError {
    NoStatesPresent,
}

impl Display for StateError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            StateError::NoStatesPresent => write!(
                fmt,
                "Tried to start state machine without any states present"
            ),
        }
    }
}

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
/// T is the type of shared data between states.
/// E is the type of custom events handled by StateEvent<E>.
pub enum Trans<T, E> {
    /// Continue as normal.
    None,
    /// Remove the active state and resume the next state on the stack or stop
    /// if there are none.
    Pop,
    /// Pause the active state and push a new state onto the stack.
    Push(Box<State<T, E>>),
    /// Remove the current state on the stack and insert a different one.
    Switch(Box<State<T, E>>),
    /// Stop and remove all states and shut down the engine.
    Quit,
}

/// An empty `Trans`. Made to be used with `EmptyState`.
pub type EmptyTrans = Trans<(), ()>;

/// A simple default `Trans`. Made to be used with `SimpleState`.
/// By default it contains a `GameData` as its `StateData` and doesn't have a custom event type.
pub type SimpleTrans<'a, 'b> = Trans<GameData<'a, 'b>, ()>;

/// A trait which defines game states that can be used by the state machine.
pub trait State<T, E: Send + Sync + 'static> {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<T>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<T>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<T>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<T>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, _data: StateData<T>, _event: StateEvent<E>) -> Trans<T, E> {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _data: StateData<T>) -> Trans<T, E> {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, _data: StateData<T>) -> Trans<T, E> {
        Trans::None
    }
}

/// An empty `State` trait. It contains no `StateData` or custom `StateEvent`.
pub trait EmptyState {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<()>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<()>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<()>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<()>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, _data: StateData<()>, event: StateEvent<()>) -> EmptyTrans {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _data: StateData<()>) -> EmptyTrans {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, _data: StateData<()>) -> EmptyTrans {
        Trans::None
    }
}

impl<T: EmptyState> State<(), ()> for T {
    /// Executed when the game state begins.
    fn on_start(&mut self, data: StateData<()>) {
        self.on_start(data)
    }

    /// Executed when the game state exits.
    fn on_stop(&mut self, data: StateData<()>) {
        self.on_stop(data)
    }

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, data: StateData<()>) {
        self.on_pause(data)
    }

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, data: StateData<()>) {
        self.on_resume(data)
    }

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, data: StateData<()>, event: StateEvent<()>) -> EmptyTrans {
        self.handle_event(data, event)
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, data: StateData<()>) -> EmptyTrans {
        self.fixed_update(data)
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, data: StateData<()>) -> EmptyTrans {
        self.update(data)
    }
}

/// A simple `State` trait. It contains `GameData` as its `StateData` and no custom `StateEvent`.
pub trait SimpleState<'a, 'b> {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<GameData>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<GameData>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<GameData>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<GameData>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(
        &mut self,
        _data: StateData<GameData>,
        event: StateEvent<()>,
    ) -> SimpleTrans<'a, 'b> {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, _data: StateData<GameData>) -> SimpleTrans<'a, 'b> {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, _data: &mut StateData<GameData>) -> SimpleTrans<'a, 'b> {
        Trans::None
    }
}
impl<'a, 'b, T: SimpleState<'a, 'b>> State<GameData<'a, 'b>, ()> for T {
    //pub trait SimpleState<'a,'b>: State<GameData<'a,'b>,()> {

    /// Executed when the game state begins.
    fn on_start(&mut self, data: StateData<GameData>) {
        self.on_start(data)
    }

    /// Executed when the game state exits.
    fn on_stop(&mut self, data: StateData<GameData>) {
        self.on_stop(data)
    }

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, data: StateData<GameData>) {
        self.on_pause(data)
    }

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, data: StateData<GameData>) {
        self.on_resume(data)
    }

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(
        &mut self,
        data: StateData<GameData>,
        event: StateEvent<()>,
    ) -> SimpleTrans<'a, 'b> {
        self.handle_event(data, event)
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, data: StateData<GameData>) -> SimpleTrans<'a, 'b> {
        self.fixed_update(data)
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, mut data: StateData<GameData>) -> SimpleTrans<'a, 'b> {
        let r = self.update(&mut data);
        data.data.update(&data.world);
        r
    }
}

/// A simple stack-based state machine (pushdown automaton).
#[derive(Derivative)]
#[derivative(Debug)]
pub struct StateMachine<'a, T, E> {
    running: bool,
    #[derivative(Debug = "ignore")]
    state_stack: Vec<Box<State<T, E> + 'a>>,
}

impl<'a, T, E: Send + Sync + 'static> StateMachine<'a, T, E> {
    /// Creates a new state machine with the given initial state.
    pub fn new<S: State<T, E> + 'a>(initial_state: S) -> StateMachine<'a, T, E> {
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
    pub fn start(&mut self, data: StateData<T>) -> Result<(), StateError> {
        if !self.running {
            let state = self
                .state_stack
                .last_mut()
                .ok_or(StateError::NoStatesPresent)?;
            state.on_start(data);
            self.running = true;
        }
        Ok(())
    }

    /// Passes a single event to the active state to handle.
    pub fn handle_event(&mut self, data: StateData<T>, event: StateEvent<E>) {
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
    fn transition(&mut self, request: Trans<T, E>, data: StateData<T>) {
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
    fn switch(&mut self, state: Box<State<T, E>>, data: StateData<T>) {
        if self.running {
            let StateData { world, data } = data;
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData { world, data });
            }

            self.state_stack.push(state);

            //State was just pushed, thus pop will always succeed
            let state = self.state_stack.last_mut().unwrap();
            state.on_start(StateData { world, data });
        }
    }

    /// Pauses the active state and pushes a new state onto the state stack.
    fn push(&mut self, state: Box<State<T, E>>, data: StateData<T>) {
        if self.running {
            let StateData { world, data } = data;
            if let Some(state) = self.state_stack.last_mut() {
                state.on_pause(StateData { world, data });
            }

            self.state_stack.push(state);

            //State was just pushed, thus pop will always succeed
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

    impl State<(), ()> for State1 {
        fn update(&mut self, _: StateData<()>) -> Trans<(), ()> {
            if self.0 > 0 {
                self.0 -= 1;
                Trans::None
            } else {
                Trans::Switch(Box::new(State2))
            }
        }
    }

    impl State<(), ()> for State2 {
        fn update(&mut self, _: StateData<()>) -> Trans<(), ()> {
            Trans::Pop
        }
    }

    #[test]
    fn switch_pop() {
        use ecs::prelude::World;

        let mut world = World::new();

        let mut sm = StateMachine::new(State1(7));
        sm.start(StateData::new(&mut world, &mut ())).unwrap();

        for _ in 0..8 {
            sm.update(StateData::new(&mut world, &mut ()));
            assert!(sm.is_running());
        }

        sm.update(StateData::new(&mut world, &mut ()));
        assert!(!sm.is_running());
    }
}
