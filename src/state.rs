//! Utilities for game state management.

use amethyst_input::is_close_requested;

use derivative::Derivative;

use crate::{ecs::prelude::World, GameData, StateEvent};

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Error type for errors occurring in StateMachine
#[derive(Debug)]
pub enum StateError {
    NoStatesPresent,
}

impl Display for StateError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        match *self {
            StateError::NoStatesPresent => write!(
                fmt,
                "Tried to start state machine without any states present"
            ),
        }
    }
}

/// State data encapsulates the data sent to all state functions from the application main loop.
pub struct StateData<'a, T> {
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
/// E is the type of events
pub enum Trans<T, E> {
    /// Continue as normal.
    None,
    /// Remove the active state and resume the next state on the stack or stop
    /// if there are none.
    Pop,
    /// Pause the active state and push a new state onto the stack.
    Push(Box<dyn State<T, E>>),
    /// Remove the current state on the stack and insert a different one.
    Switch(Box<dyn State<T, E>>),
    /// Stop and remove all states and shut down the engine.
    Quit,
}

/// Event queue to trigger state `Trans` from other places than a `State`'s methods.
/// # Example:
/// ```rust, ignore
/// world.write_resource::<EventChannel<TransEvent<MyGameData, StateEvent>>>().single_write(Box::new(|| Trans::Quit));
/// ```
///
/// Transitions will be executed sequentially by Amethyst's `CoreApplication` update loop.
pub type TransEvent<T, E> = Box<dyn Fn() -> Trans<T, E> + Send + Sync + 'static>;

/// An empty `Trans`. Made to be used with `EmptyState`.
pub type EmptyTrans = Trans<(), StateEvent>;

/// A simple default `Trans`. Made to be used with `SimpleState`.
/// By default it contains a `GameData` as its `StateData` and doesn't have a custom event type.
pub type SimpleTrans = Trans<GameData<'static, 'static>, StateEvent>;

/// A trait which defines game states that can be used by the state machine.
pub trait State<T, E: Send + Sync + 'static> {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<'_, T>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<'_, T>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<'_, T>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<'_, T>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, _data: StateData<'_, T>, _event: E) -> Trans<T, E> {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// if this is the active state.
    fn fixed_update(&mut self, _data: StateData<'_, T>) -> Trans<T, E> {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// if this is the active state.
    fn update(&mut self, _data: StateData<'_, T>) -> Trans<T, E> {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, _data: StateData<'_, T>) {}

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, _data: StateData<'_, T>) {}
}

/// An empty `State` trait. It contains no `StateData` or custom `StateEvent`.
pub trait EmptyState {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<'_, ()>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<'_, ()>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<'_, ()>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<'_, ()>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, _data: StateData<'_, ()>, event: StateEvent) -> EmptyTrans {
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
    fn fixed_update(&mut self, _data: StateData<'_, ()>) -> EmptyTrans {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, _data: StateData<'_, ()>) -> EmptyTrans {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, _data: StateData<'_, ()>) {}

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, _data: StateData<'_, ()>) {}
}

impl<T: EmptyState> State<(), StateEvent> for T {
    /// Executed when the game state begins.
    fn on_start(&mut self, data: StateData<'_, ()>) {
        self.on_start(data)
    }

    /// Executed when the game state exits.
    fn on_stop(&mut self, data: StateData<'_, ()>) {
        self.on_stop(data)
    }

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, data: StateData<'_, ()>) {
        self.on_pause(data)
    }

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, data: StateData<'_, ()>) {
        self.on_resume(data)
    }

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, data: StateData<'_, ()>, event: StateEvent) -> EmptyTrans {
        self.handle_event(data, event)
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, data: StateData<'_, ()>) -> EmptyTrans {
        self.fixed_update(data)
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, data: StateData<'_, ()>) -> EmptyTrans {
        self.update(data)
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, data: StateData<'_, ()>) {
        self.shadow_fixed_update(data);
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, data: StateData<'_, ()>) {
        self.shadow_update(data);
    }
}

/// A simple `State` trait. It contains `GameData` as its `StateData` and no custom `StateEvent`.
pub trait SimpleState {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
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
    fn fixed_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}
}

impl<T: SimpleState> State<GameData<'static, 'static>, StateEvent> for T {
    //pub trait SimpleState<'a,'b>: State<GameData<'a,'b>,()> {

    /// Executed when the game state begins.
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.on_start(data)
    }

    /// Executed when the game state exits.
    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.on_stop(data)
    }

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.on_pause(data)
    }

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.on_resume(data)
    }

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        self.handle_event(data, event)
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        self.fixed_update(data)
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, mut data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let r = self.update(&mut data);
        data.data.update(&data.world);
        r
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.shadow_fixed_update(data);
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.shadow_update(data);
    }
}

/// A simple stack-based state machine (pushdown automaton).
#[derive(Derivative)]
#[derivative(Debug)]
pub struct StateMachine<'a, T, E> {
    running: bool,
    #[derivative(Debug = "ignore")]
    state_stack: Vec<Box<dyn State<T, E> + 'a>>,
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
    pub fn start(&mut self, data: StateData<'_, T>) -> Result<(), StateError> {
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
    pub fn handle_event(&mut self, data: StateData<'_, T>, event: E) {
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
    pub fn fixed_update(&mut self, data: StateData<'_, T>) {
        let StateData { world, data } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.fixed_update(StateData { world, data }),
                None => Trans::None,
            };
            for state in self.state_stack.iter_mut() {
                state.shadow_fixed_update(StateData { world, data });
            }

            self.transition(trans, StateData { world, data });
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, data: StateData<'_, T>) {
        let StateData { world, data } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.update(StateData { world, data }),
                None => Trans::None,
            };
            for state in self.state_stack.iter_mut() {
                state.shadow_update(StateData { world, data });
            }

            self.transition(trans, StateData { world, data });
        }
    }

    /// Performs a state transition.
    /// Usually called by update or fixed_update by the user's defined `State`.
    /// This method can also be called when there are one or multiple `Trans` stored in the
    /// global `EventChannel<TransEvent<T, E>>`. Such `Trans` will be passed to this method
    /// sequentially in the order of insertion.
    pub fn transition(&mut self, request: Trans<T, E>, data: StateData<'_, T>) {
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
    fn switch(&mut self, state: Box<dyn State<T, E>>, data: StateData<'_, T>) {
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
    fn push(&mut self, state: Box<dyn State<T, E>>, data: StateData<'_, T>) {
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
    fn pop(&mut self, data: StateData<'_, T>) {
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
    pub(crate) fn stop(&mut self, data: StateData<'_, T>) {
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
        fn update(&mut self, _: StateData<'_, ()>) -> Trans<(), ()> {
            if self.0 > 0 {
                self.0 -= 1;
                Trans::None
            } else {
                Trans::Switch(Box::new(State2))
            }
        }
    }

    impl State<(), ()> for State2 {
        fn update(&mut self, _: StateData<'_, ()>) -> Trans<(), ()> {
            Trans::Pop
        }
    }

    #[test]
    fn switch_pop() {
        use crate::ecs::prelude::World;

        let mut world = World::new();

        let mut sm = StateMachine::new(State1(7));
        // Unwrap here is fine because start can only fail when there are no states in the machine.
        sm.start(StateData::new(&mut world, &mut ())).unwrap();

        for _ in 0..8 {
            sm.update(StateData::new(&mut world, &mut ()));
            assert!(sm.is_running());
        }

        sm.update(StateData::new(&mut world, &mut ()));
        assert!(!sm.is_running());
    }
}
