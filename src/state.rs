//! Utilities for game state management.

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

use amethyst_input::is_close_requested;
use derivative::Derivative;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{ecs::*, GameData, StateEvent};

/// Error type for errors occurring in `StateMachine`
#[derive(Debug)]
pub enum StateError {
    NoStatesPresent,
}

impl Display for StateError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        match *self {
            StateError::NoStatesPresent => {
                write!(
                    fmt,
                    "Tried to start state machine without any states present"
                )
            }
        }
    }
}

/// State data encapsulates the data sent to all state functions from the application main loop.
#[allow(missing_debug_implementations)]
pub struct StateData<'a, T> {
    /// Main `World`
    pub world: &'a mut World,
    /// Resources
    pub resources: &'a mut Resources,
    /// User defined game data
    pub data: &'a mut T,
}

impl<'a, T> StateData<'a, T>
where
    T: 'a,
{
    /// Create a new state data
    pub fn new(world: &'a mut World, resources: &'a mut Resources, data: &'a mut T) -> Self {
        StateData {
            world,
            resources,
            data,
        }
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
    /// Remove all states on the stack and insert a different one.
    Replace(Box<dyn State<T, E>>),
    /// Remove all states on the stack and insert new stack.
    NewStack(Vec<Box<dyn State<T, E>>>),
    /// Execute a series of Trans's.
    Sequence(Vec<Trans<T, E>>),
    /// Stop and remove all states and shut down the engine.
    Quit,
}
impl<T, E> Debug for Trans<T, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Trans::None => f.write_str("None"),
            Trans::Pop => f.write_str("Pop"),
            Trans::Push(_) => f.write_str("Push"),
            Trans::Switch(_) => f.write_str("Switch"),
            Trans::Replace(_) => f.write_str("Replace"),
            Trans::NewStack(_) => f.write_str("NewStack"),
            Trans::Sequence(sequence) => f.write_str(&format!("Sequence {:?}", sequence)),
            Trans::Quit => f.write_str("Quit"),
        }
    }
}

/// Event queue to trigger state `Trans` from other places than a `State`'s methods.
/// FIXME: needs example
/// # Example:
/// ```ignore
/// resources.get_mut::<EventChannel<TransEvent<MyGameData, StateEvent>>>().single_write(Box::new(|| Trans::Quit));
/// ```
///
/// Transitions will be executed sequentially by Amethyst's `CoreApplication` update loop.
pub type TransEvent<T, E> = Box<dyn Fn() -> Trans<T, E> + Send + Sync + 'static>;

/// An empty `Trans`. Made to be used with `EmptyState`.
pub type EmptyTrans = Trans<(), StateEvent>;

/// A simple default `Trans`. Made to be used with `SimpleState`.
/// By default it contains a `GameData` as its `StateData` and doesn't have a custom event type.
pub type SimpleTrans = Trans<GameData, StateEvent>;

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
    fn on_start(&mut self, _data: StateData<'_, GameData>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<'_, GameData>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<'_, GameData>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<'_, GameData>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, _data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
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
    fn fixed_update(&mut self, _data: StateData<'_, GameData>) -> SimpleTrans {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, _data: &mut StateData<'_, GameData>) -> SimpleTrans {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, _data: StateData<'_, GameData>) {}

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, _data: StateData<'_, GameData>) {}
}

impl<T: SimpleState> State<GameData, StateEvent> for T {
    /// Executed when the game state begins.
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        self.on_start(data)
    }

    /// Executed when the game state exits.
    fn on_stop(&mut self, data: StateData<'_, GameData>) {
        self.on_stop(data)
    }

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, data: StateData<'_, GameData>) {
        self.on_pause(data)
    }

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, data: StateData<'_, GameData>) {
        self.on_resume(data)
    }

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        self.handle_event(data, event)
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default).
    fn fixed_update(&mut self, data: StateData<'_, GameData>) -> SimpleTrans {
        self.fixed_update(data)
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit).
    fn update(&mut self, mut data: StateData<'_, GameData>) -> SimpleTrans {
        let r = self.update(&mut data);
        data.data.update(&mut data.world, &mut data.resources);
        r
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, data: StateData<'_, GameData>) {
        self.shadow_fixed_update(data);
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, data: StateData<'_, GameData>) {
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
        let StateData {
            world,
            resources,
            data,
        } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => {
                    state.handle_event(
                        StateData {
                            world,
                            resources,
                            data,
                        },
                        event,
                    )
                }
                None => Trans::None,
            };

            self.transition(
                trans,
                StateData {
                    world,
                    resources,
                    data,
                },
            );
        }
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, data: StateData<'_, T>) {
        let StateData {
            world,
            resources,
            data,
        } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => {
                    #[cfg(feature = "profiler")]
                    profile_scope!("stack fixed_update");
                    state.fixed_update(StateData {
                        world,
                        resources,
                        data,
                    })
                }
                None => Trans::None,
            };
            for state in &mut self.state_stack {
                #[cfg(feature = "profiler")]
                profile_scope!("stack shadow_fixed_update");
                state.shadow_fixed_update(StateData {
                    world,
                    resources,
                    data,
                });
            }
            {
                #[cfg(feature = "profiler")]
                profile_scope!("stack fixed transition");
                self.transition(
                    trans,
                    StateData {
                        world,
                        resources,
                        data,
                    },
                );
            }
        }
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, data: StateData<'_, T>) {
        let StateData {
            world,
            resources,
            data,
        } = data;
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => {
                    #[cfg(feature = "profiler")]
                    profile_scope!("stack update");
                    state.update(StateData {
                        world,
                        resources,
                        data,
                    })
                }
                None => Trans::None,
            };
            for state in &mut self.state_stack {
                #[cfg(feature = "profiler")]
                profile_scope!("stack shadow_update");
                state.shadow_update(StateData {
                    world,
                    resources,
                    data,
                });
            }

            {
                #[cfg(feature = "profiler")]
                profile_scope!("stack transition");
                self.transition(
                    trans,
                    StateData {
                        world,
                        resources,
                        data,
                    },
                );
            }
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
                Trans::Replace(state) => self.replace(state, data),
                Trans::NewStack(states) => self.new_stack(states, data),
                Trans::Sequence(sequence) => {
                    for trans in sequence {
                        let temp_data = StateData {
                            world: data.world,
                            resources: data.resources,
                            data: data.data,
                        };
                        self.transition(trans, temp_data);
                    }
                }
                Trans::Quit => self.stop(data),
            }
        }
    }

    /// Removes the current state on the stack and inserts a different one.
    fn switch(&mut self, state: Box<dyn State<T, E>>, data: StateData<'_, T>) {
        if self.running {
            let StateData {
                world,
                resources,
                data,
            } = data;
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData {
                    world,
                    resources,
                    data,
                });
            }

            self.state_stack.push(state);

            //State was just pushed, thus pop will always succeed
            let new_state = self.state_stack.last_mut().unwrap();
            new_state.on_start(StateData {
                world,
                resources,
                data,
            });
        }
    }

    /// Pauses the active state and pushes a new state onto the state stack.
    fn push(&mut self, state: Box<dyn State<T, E>>, data: StateData<'_, T>) {
        if self.running {
            let StateData {
                world,
                resources,
                data,
            } = data;
            if let Some(state) = self.state_stack.last_mut() {
                state.on_pause(StateData {
                    world,
                    resources,
                    data,
                });
            }

            self.state_stack.push(state);

            //State was just pushed, thus pop will always succeed
            let new_state = self.state_stack.last_mut().unwrap();
            new_state.on_start(StateData {
                world,
                resources,
                data,
            });
        }
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    fn pop(&mut self, data: StateData<'_, T>) {
        if self.running {
            let StateData {
                world,
                resources,
                data,
            } = data;
            if let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData {
                    world,
                    resources,
                    data,
                });
            }

            if let Some(state) = self.state_stack.last_mut() {
                state.on_resume(StateData {
                    world,
                    resources,
                    data,
                });
            } else {
                self.running = false;
            }
        }
    }

    /// Removes all states from the stack and replaces it with a new state.
    pub(crate) fn replace(&mut self, state: Box<dyn State<T, E>>, data: StateData<'_, T>) {
        if self.running {
            //Pemove all current states
            let StateData {
                world,
                resources,
                data,
            } = data;
            while let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData {
                    world,
                    resources,
                    data,
                });
            }

            //Push the new state
            self.state_stack.push(state);

            //State was just pushed, thus pop will always succeed
            let new_state = self.state_stack.last_mut().unwrap();
            new_state.on_start(StateData {
                world,
                resources,
                data,
            });
        }
    }

    /// Removes all states from the stack and replaces it with a new stack.
    pub(crate) fn new_stack(&mut self, states: Vec<Box<dyn State<T, E>>>, data: StateData<'_, T>) {
        if self.running {
            //remove all current states
            let StateData {
                world,
                resources,
                data,
            } = data;
            while let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData {
                    world,
                    resources,
                    data,
                });
            }

            // push the new states
            let state_count = states.len();
            for (count, state) in states.into_iter().enumerate() {
                self.state_stack.push(state);

                // State was just pushed, thus pop will always succeed
                let new_state = self.state_stack.last_mut().unwrap();
                new_state.on_start(StateData {
                    world,
                    resources,
                    data,
                });
                if count != state_count - 1 {
                    // pause on each state but the last
                    new_state.on_pause(StateData {
                        world,
                        resources,
                        data,
                    });
                }
            }
        }
    }

    /// Shuts the state machine down.
    pub(crate) fn stop(&mut self, data: StateData<'_, T>) {
        if self.running {
            let StateData {
                world,
                resources,
                data,
            } = data;
            while let Some(mut state) = self.state_stack.pop() {
                state.on_stop(StateData {
                    world,
                    resources,
                    data,
                });
            }

            self.running = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct State0;
    struct State1(u8);
    struct State2;
    struct StateNewStack;
    struct StateSequence;
    struct StateReplace(u8);

    impl State<(), ()> for State0 {
        fn update(&mut self, _: StateData<'_, ()>) -> Trans<(), ()> {
            Trans::None
        }
    }

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

    impl State<(), ()> for StateNewStack {
        fn update(&mut self, _: StateData<'_, ()>) -> Trans<(), ()> {
            Trans::NewStack(vec![
                Box::new(State0),
                Box::new(State0),
                Box::new(State0),
                Box::new(State0),
            ])
        }
    }
    impl State<(), ()> for StateSequence {
        fn update(&mut self, _: StateData<'_, ()>) -> Trans<(), ()> {
            Trans::Sequence(vec![
                Trans::Push(Box::new(State0)),
                Trans::Push(Box::new(State0)),
                Trans::Push(Box::new(State0)),
                Trans::Pop,
            ])
        }
    }
    impl State<(), ()> for StateReplace {
        fn update(&mut self, _: StateData<'_, ()>) -> Trans<(), ()> {
            if self.0 == 0 {
                Trans::Replace(Box::new(State0))
            } else {
                Trans::Push(Box::new(StateReplace(self.0 - 1)))
            }
        }
    }

    #[test]
    fn switch_pop() {
        use crate::ecs::World;

        let mut world = World::default();
        let mut resources = Resources::default();

        let mut sm = StateMachine::new(State1(7));
        // Unwrap here is fine because start can only fail when there are no states in the machine.
        sm.start(StateData::new(&mut world, &mut resources, &mut ()))
            .unwrap();

        for _ in 0..8 {
            sm.update(StateData::new(&mut world, &mut resources, &mut ()));
            assert!(sm.is_running());
        }

        sm.update(StateData::new(&mut world, &mut resources, &mut ()));
        assert!(!sm.is_running());
    }

    #[test]
    fn new_stack() {
        use crate::ecs::World;

        let mut world = World::default();
        let mut resources = Resources::default();

        let mut sm = StateMachine::new(StateNewStack);
        // Unwrap here is fine because start can only fail when there are no states in the machine.
        sm.start(StateData::new(&mut world, &mut resources, &mut ()))
            .unwrap();

        sm.update(StateData::new(&mut world, &mut resources, &mut ()));
        assert_eq!(sm.state_stack.len(), 4);
    }

    #[test]
    fn sequence() {
        use crate::ecs::World;

        let mut world = World::default();
        let mut resources = Resources::default();

        let mut sm = StateMachine::new(StateSequence);
        // Unwrap here is fine because start can only fail when there are no states in the machine.
        sm.start(StateData::new(&mut world, &mut resources, &mut ()))
            .unwrap();

        sm.update(StateData::new(&mut world, &mut resources, &mut ()));
        assert_eq!(sm.state_stack.len(), 3);
    }

    #[test]
    fn replace() {
        use crate::ecs::World;

        let mut world = World::default();
        let mut resources = Resources::default();

        let mut sm = StateMachine::new(StateReplace(3));
        // Unwrap here is fine because start can only fail when there are no states in the machine.
        sm.start(StateData::new(&mut world, &mut resources, &mut ()))
            .unwrap();

        for i in 0..3 {
            sm.update(StateData::new(&mut world, &mut resources, &mut ()));
            assert_eq!(sm.state_stack.len(), i + 2);
        }

        sm.update(StateData::new(&mut world, &mut resources, &mut ()));
        assert_eq!(sm.state_stack.len(), 1);
    }
}
