//! Dynamic utilities for game state management.

use crate::{ecs::prelude::World, error::Error, Trans};

use hashbrown;
use smallvec::SmallVec;
use std::fmt;
use std::hash;
use std::mem;

/// The type of a new state.
pub(crate) type NewState<S, E> = (S, Box<dyn StateCallback<S, E> + Send + Sync>);

/// A resource used to indirectly manipulate the contents of the state machine.
pub struct States<S, E> {
    new_states: SmallVec<[NewState<S, E>; 16]>,
}

impl<S, E> Default for States<S, E> {
    fn default() -> Self {
        States {
            new_states: Default::default(),
        }
    }
}

impl<S, E> States<S, E> {
    /// Indicate that we want to create a new state.
    pub fn new_state<C>(&mut self, state: S, callback: C)
    where
        C: 'static + StateCallback<S, E> + Send + Sync,
    {
        self.new_states.push((state, Box::new(callback)));
    }

    /// Drain all new states and push into the provided callback.
    pub fn drain_new_states<C>(&mut self, mut c: C)
    where
        C: FnMut(NewState<S, E>),
    {
        for new_state in self.new_states.drain() {
            c(new_state)
        }
    }
}

/// Error type for errors occurring in StateMachine
#[derive(Debug)]
pub enum StateError {
    /// Error raised when no states are present.
    NoStatesPresent,
    /// Conflicting handler for state.
    CallbackConflict,
}

impl fmt::Display for StateError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            StateError::NoStatesPresent => write!(
                fmt,
                "Tried to start state machine without any states present"
            ),
            StateError::CallbackConflict => {
                write!(fmt, "Tried to register conflicting callback for state.")
            }
        }
    }
}

/// The trait associated with a stage.
pub trait State<E>: Clone + Default + fmt::Debug
where
    Self: Sized,
{
    /// The storage used for storing callbacks for the given state.
    type Storage: Default + StateStorage<Self, E>;
}

/// Provides access to storage for states.
pub trait StateStorage<S, E>
where
    Self: Sized,
{
    /// Insert the given callback, returning an existing callback if it is already present.
    fn insert(
        &mut self,
        state: S,
        callback: Box<dyn StateCallback<S, E>>,
    ) -> Option<Box<dyn StateCallback<S, E>>>;

    /// Get mutable storage for the given state.
    fn get_mut(&mut self, value: &S) -> Option<&mut Box<dyn StateCallback<S, E>>>;

    /// Apply the specified function to all values.
    fn do_values<F>(&mut self, apply: F)
    where
        F: FnMut(&mut Box<dyn StateCallback<S, E>>);
}

/// A callback that is registered for all events.
/// This is typically used for bookkeeping specific things.
pub trait GlobalCallback<S, E> {
    /// Fired when state machine has been started.
    fn started(&mut self, _: &mut World) {}

    /// Fired when state machine has been stopped.
    fn stopped(&mut self, _: &mut World) {}

    /// Fired when state has changed, and what it was changed to.
    fn changed(&mut self, _: &mut World, _: &S) {}

    /// Fired on events.
    ///
    /// If multiple callbacks would result in a state transition, they will be applied one after
    /// another in an undetermined order.
    fn handle_event(&mut self, _: &mut World, _: &E) -> Trans<S> {
        Trans::None
    }

    /// Fired on fixed updates.
    fn fixed_update(&mut self, _: &mut World) -> Trans<S> {
        Trans::None
    }

    /// Fired on updates.
    fn update(&mut self, _: &mut World) -> Trans<S> {
        Trans::None
    }
}

/// A callback that is registered for a specific state.
pub trait StateCallback<S, E> {
    /// Executed when the game state begins.
    fn on_start(&mut self, _: &mut World) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _: &mut World) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _: &mut World) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _: &mut World) {}

    /// Fired on events.
    fn handle_event(&mut self, _: &mut World, _: &E) -> Trans<S> {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// if this is the active state.
    fn fixed_update(&mut self, _: &mut World) -> Trans<S> {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// if this is the active state.
    fn update(&mut self, _: &mut World) -> Trans<S> {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second
    /// by default),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, _: &mut World) -> Trans<S> {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow (taking into account the frame rate limit),
    /// even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, _: &mut World) -> Trans<S> {
        Trans::None
    }
}

/// How many callbacks that are inlined.
const INLINED_CALLBACKS: usize = 16;

/// Type alias for a collection of global callbacks.
type GlobalCallbacks<S, E> = SmallVec<[Box<dyn GlobalCallback<S, E>>; INLINED_CALLBACKS]>;

/// A simple stack-based state machine (pushdown automaton).
#[derive(Derivative)]
#[derivative(Debug)]
pub struct StateMachine<S, E>
where
    S: State<E>,
{
    running: bool,
    /// The stack of the state machine.
    #[derivative(Debug = "ignore")]
    stack: Vec<S>,
    /// Callbacks fired on particular states.
    #[derivative(Debug = "ignore")]
    callbacks: S::Storage,
    /// Callbacks fired on any state.
    #[derivative(Debug = "ignore")]
    global_callbacks: GlobalCallbacks<S, E>,
}

impl<S, E> StateMachine<S, E>
where
    S: State<E>,
{
    /// Creates a new state machine with the given initial state.
    pub fn new() -> StateMachine<S, E> {
        StateMachine {
            running: false,
            stack: vec![],
            callbacks: Default::default(),
            global_callbacks: Default::default(),
        }
    }

    /// Register a callback associated with a specific state.
    pub fn register_callback<C: 'static>(&mut self, state: S, callback: C) -> Result<(), Error>
    where
        C: StateCallback<S, E>,
    {
        self.register_boxed_callback(state, Box::new(callback))
    }

    /// Register a callback associated with a specific state.
    pub fn register_boxed_callback(
        &mut self,
        state: S,
        callback: Box<dyn StateCallback<S, E>>,
    ) -> Result<(), Error> {
        if self.callbacks.insert(state, callback).is_some() {
            return Err(Error::StateMachine(StateError::CallbackConflict));
        }

        Ok(())
    }

    /// Register a callback associated with a specific state at runtime.
    ///
    /// This forcibly overrides any existing states and makes sure that the proper callbacks are
    /// invoked.
    pub fn runtime_register_boxed_callback(
        &mut self,
        state: S,
        mut callback: Box<dyn StateCallback<S, E>>,
        world: &mut World,
    ) {
        callback.on_start(world);

        if let Some(mut old) = self.callbacks.insert(state, callback) {
            old.on_stop(world);
        }
    }

    /// Register a global callback that is called on any state.
    ///
    /// A global callback received "global" events for this state machine, this includes:
    ///
    /// * When it is started.
    /// * When it is stopped.
    /// * When it changes state.
    /// * When it received an event.
    /// * When it received an update.
    /// * When it received a fixed update.
    ///
    /// This is done through the [`GlobalCallback`](trait.GlobalCallback.html) trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate amethyst;
    ///
    /// use amethyst::{ecs::World, StateMachine, Trans, GlobalCallback};
    /// use std::{rc::Rc, cell::RefCell};
    ///
    /// #[derive(State, Clone, Debug)]
    /// enum State {
    ///     First,
    ///     Second,
    /// }
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq)]
    /// struct Event(&'static str);
    ///
    /// struct Global {
    ///     capture: Rc<RefCell<Vec<String>>>,
    /// }
    ///
    /// impl GlobalCallback<State, Event> for Global {
    ///     fn started(&mut self, world: &mut World) {
    ///         self.capture.borrow_mut().push(format!("Started"));
    ///     }
    ///
    ///     fn stopped(&mut self, world: &mut World) {
    ///         self.capture.borrow_mut().push(format!("Stopped"));
    ///     }
    ///
    ///     fn changed(&mut self, world: &mut World, state: &State) {
    ///         self.capture.borrow_mut().push(format!("Changed: {:?}", state));
    ///     }
    ///
    ///     fn handle_event(&mut self, world: &mut World, event: &Event) -> Trans<State> {
    ///         self.capture.borrow_mut().push(format!("Event: {}", event.0));
    ///         Trans::None
    ///     }
    ///
    ///     fn update(&mut self, world: &mut World) -> Trans<State> {
    ///         self.capture.borrow_mut().push(format!("Update"));
    ///         Trans::None
    ///     }
    ///
    ///     fn fixed_update(&mut self, world: &mut World) -> Trans<State> {
    ///         self.capture.borrow_mut().push(format!("Fixed Update"));
    ///         Trans::None
    ///     }
    /// }
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// let mut states = StateMachine::new();
    /// // Helper to capture events.
    /// let capture = Rc::new(RefCell::new(Vec::new()));
    /// // The world
    /// let mut world = World::new();
    ///
    /// // Set up the global event handler with our capture.
    /// states.register_global(Global {
    ///     capture: Rc::clone(&capture),
    /// });
    ///
    /// states.start(&mut world);
    /// states.handle_event(&mut world, Event("Message"));
    ///
    /// states.update(&mut world);
    ///
    /// states.transition(Trans::Switch(State::Second), &mut world);
    ///
    /// states.fixed_update(&mut world);
    ///
    /// // Note: popping the last state stops the state machine.
    /// states.transition(Trans::Pop, &mut world);
    ///
    /// assert_eq!(
    ///     *capture.borrow(),
    ///     &[
    ///         "Started",
    ///         "Event: Message",
    ///         "Update",
    ///         "Changed: Second",
    ///         "Fixed Update",
    ///         "Stopped",
    ///     ]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn register_global<C: 'static>(&mut self, callback: C)
    where
        C: GlobalCallback<S, E>,
    {
        self.register_boxed_global(Box::new(callback));
    }

    /// Register a boxed global callback that is called on any state.
    pub fn register_boxed_global(&mut self, callback: Box<dyn GlobalCallback<S, E>>) {
        self.global_callbacks.push(callback);
    }

    /// Checks whether the state machine is running.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate amethyst;
    /// use amethyst::{ecs::World, StateMachine};
    ///
    /// #[derive(State, Clone, Debug)]
    /// enum State {
    ///     First,
    ///     Second,
    /// }
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// let mut states = StateMachine::<State, ()>::new();
    /// // the world
    /// let mut world = World::new();
    ///
    /// assert!(!states.is_running());
    /// states.start(&mut world);
    /// assert!(states.is_running());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Initializes the state machine.
    pub fn start(&mut self, world: &mut World) -> Result<(), Error> {
        if self.running {
            return Ok(());
        }

        let state = S::default();

        if let Some(c) = self.callbacks.get_mut(&state) {
            c.on_start(world);
        }

        for c in &mut self.global_callbacks {
            c.started(world);
        }

        self.running = true;
        self.stack.push(state);
        Ok(())
    }

    /// Passes a single event to the active state to handle.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate amethyst;
    ///
    /// use amethyst::{ecs::World, StateMachine, Trans, StateCallback};
    /// use std::{rc::Rc, cell::RefCell};
    ///
    /// #[derive(State, Debug, Clone)]
    /// enum State {
    ///     First,
    ///     Second,
    /// }
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq)]
    /// struct Event(&'static str);
    ///
    /// struct SecondState(Rc<RefCell<Option<Event>>>);
    ///
    /// impl StateCallback<State, Event> for SecondState {
    ///     fn handle_event(&mut self, world: &mut World, event: &Event) -> Trans<State> {
    ///         *self.0.borrow_mut() = Some(event.clone());
    ///         Trans::None
    ///     }
    /// }
    ///
    /// # fn main() -> amethyst::Result<()> {
    /// let mut states = StateMachine::new();
    /// // Helper to capture the event.
    /// let capture = Rc::new(RefCell::new(None));
    /// // The world
    /// let mut world = World::new();
    ///
    /// states.register_callback(State::Second, SecondState(Rc::clone(&capture)))?;
    /// states.start(&mut world);
    /// assert_eq!(*capture.borrow(), None);
    ///
    /// // Nothing happen because no callback is registered for the initial state.
    /// states.handle_event(&mut world, Event("hello"));
    /// assert_eq!(*capture.borrow(), None);
    ///
    /// // Transition to second state and verify that we've handled the event.
    /// states.transition(Trans::Push(State::Second), &mut world);
    /// states.handle_event(&mut world, Event("world"));
    /// assert_eq!(*capture.borrow(), Some(Event("world")));
    /// # Ok(())
    /// # }
    /// ```
    pub fn handle_event(&mut self, world: &mut World, event: E) {
        if !self.running {
            return;
        }

        // Transition which have been requested by callbacks.
        let mut trans = Trans::None;

        {
            let StateMachine {
                ref mut stack,
                ref mut callbacks,
                ref mut global_callbacks,
                ..
            } = *self;

            for c in global_callbacks {
                trans.update(c.handle_event(world, &event));
            }

            if let Some(c) = stack.last().and_then(|s| callbacks.get_mut(s)) {
                trans.update(c.handle_event(world, &event));
            }
        }

        self.transition(trans, world);
    }

    /// Updates the currently active state at a steady, fixed interval.
    pub fn fixed_update(&mut self, world: &mut World) {
        if !self.running {
            return;
        }

        // Transition which have been requested by callbacks.
        let mut trans = Trans::None;

        {
            let StateMachine {
                ref mut stack,
                ref mut callbacks,
                ref mut global_callbacks,
                ..
            } = *self;

            for c in global_callbacks {
                trans.update(c.fixed_update(world));
            }

            // Fixed shadow updates for all states.
            callbacks.do_values(|c| {
                trans.update(c.shadow_fixed_update(world));
            });

            if let Some(c) = stack.last().and_then(|s| callbacks.get_mut(s)) {
                trans.update(c.fixed_update(world));
            }
        }

        self.transition(trans, world);
    }

    /// Updates the currently active state immediately.
    pub fn update(&mut self, world: &mut World) {
        if !self.running {
            return;
        }

        // Transition which have been requested by callbacks.
        let mut trans = Trans::None;

        {
            let StateMachine {
                ref mut stack,
                ref mut callbacks,
                ref mut global_callbacks,
                ..
            } = *self;

            // Regular update for global callbacks.
            for c in global_callbacks {
                trans.update(c.update(world));
            }

            // Shadow updates for all states.
            callbacks.do_values(|c| {
                trans.update(c.shadow_update(world));
            });

            if let Some(c) = stack.last().and_then(|s| callbacks.get_mut(&s)) {
                trans.update(c.update(world));
            }
        }

        self.transition(trans, world);
    }

    /// Performs a state transition.
    pub fn transition(&mut self, request: Trans<S>, world: &mut World) {
        if !self.running {
            return;
        }

        match request {
            Trans::None => (),
            Trans::Pop => self.pop(world),
            Trans::Push(state) => self.push(state, world),
            Trans::Switch(state) => self.switch(state, world),
            Trans::Quit => self.stop(world),
        }
    }

    /// Removes the current state on the stack and inserts a different one.
    fn switch(&mut self, state: S, world: &mut World) {
        if !self.running {
            return;
        }

        if let Some(c) = self.stack.pop().and_then(|s| self.callbacks.get_mut(&s)) {
            c.on_stop(world);
        }

        if let Some(c) = self.callbacks.get_mut(&state) {
            c.on_start(world);
        }

        for c in &mut self.global_callbacks {
            c.changed(world, &state);
        }

        self.stack.push(state);
    }

    /// Pauses the active state and pushes a new state onto the state stack.
    fn push(&mut self, state: S, world: &mut World) {
        if !self.running {
            return;
        }

        let StateMachine {
            ref mut stack,
            ref mut callbacks,
            ..
        } = *self;

        if let Some(c) = stack.last().and_then(|s| callbacks.get_mut(&s)) {
            c.on_pause(world);
        }

        if let Some(c) = callbacks.get_mut(&state) {
            c.on_start(world);
        }

        for c in &mut self.global_callbacks {
            c.changed(world, &state);
        }

        stack.push(state);
    }

    /// Stops and removes the active state and un-pauses the next state on the
    /// stack (if any).
    fn pop(&mut self, world: &mut World) {
        if !self.running {
            return;
        }

        let head = match self.stack.pop() {
            Some(head) => head,
            None => return,
        };

        if let Some(c) = self.callbacks.get_mut(&head) {
            c.on_stop(world);
        }

        match self.stack.last() {
            Some(current) => {
                if let Some(c) = self.callbacks.get_mut(current) {
                    c.on_resume(world);
                }

                for c in &mut self.global_callbacks {
                    c.changed(world, current);
                }
            }
            None => {
                self.running = false;

                for c in &mut self.global_callbacks {
                    c.stopped(world);
                }
            }
        }
    }

    /// Shuts the state machine down.
    pub(crate) fn stop(&mut self, world: &mut World) {
        if !self.running {
            return;
        }

        while let Some(c) = self.stack.pop().and_then(|s| self.callbacks.get_mut(&s)) {
            c.on_stop(world);
        }

        for c in &mut self.global_callbacks {
            c.stopped(world);
        }

        self.running = false;
    }
}

/// Storage implementation for types which only have one value.
pub struct SingletonStateStorage<S, E> {
    callback: Option<Box<dyn StateCallback<S, E>>>,
}

impl<S, E> Default for SingletonStateStorage<S, E> {
    fn default() -> Self {
        SingletonStateStorage { callback: None }
    }
}

impl<S, E> StateStorage<S, E> for SingletonStateStorage<S, E> {
    fn insert(
        &mut self,
        _: S,
        callback: Box<dyn StateCallback<S, E>>,
    ) -> Option<Box<dyn StateCallback<S, E>>> {
        mem::replace(&mut self.callback, Some(callback))
    }

    fn get_mut(&mut self, _: &S) -> Option<&mut Box<dyn StateCallback<S, E>>> {
        self.callback.as_mut()
    }

    fn do_values<F>(&mut self, mut apply: F)
    where
        F: FnMut(&mut Box<dyn StateCallback<S, E>>),
    {
        if let Some(c) = self.callback.as_mut() {
            apply(c);
        }
    }
}

/// Storage implementation for types which can be hashed.
pub struct MapStateStorage<S, E>
where
    S: hash::Hash + PartialEq + Eq,
{
    callbacks: hashbrown::HashMap<S, Box<dyn StateCallback<S, E>>>,
}

impl<S, E> StateStorage<S, E> for MapStateStorage<S, E>
where
    S: hash::Hash + PartialEq + Eq,
{
    fn insert(
        &mut self,
        state: S,
        callback: Box<dyn StateCallback<S, E>>,
    ) -> Option<Box<dyn StateCallback<S, E>>> {
        self.callbacks.insert(state, callback)
    }

    fn get_mut(&mut self, state: &S) -> Option<&mut Box<dyn StateCallback<S, E>>> {
        self.callbacks.get_mut(state)
    }

    fn do_values<F>(&mut self, mut apply: F)
    where
        F: FnMut(&mut Box<dyn StateCallback<S, E>>),
    {
        for c in self.callbacks.values_mut() {
            apply(c);
        }
    }
}

impl<S, E> Default for MapStateStorage<S, E>
where
    S: hash::Hash + PartialEq + Eq,
{
    fn default() -> Self {
        MapStateStorage {
            callbacks: hashbrown::HashMap::new(),
        }
    }
}

impl<E> State<E> for () {
    type Storage = SingletonStateStorage<Self, E>;
}

/// Helper macro to implement storage for certain types.
macro_rules! impl_map_storage {
    ($lt:lifetime, $ty:ty) => {
        impl<$lt, E> State<E> for &$lt $ty {
            type Storage = MapStateStorage<Self, E>;
        }
    };

    ($ty:ty) => {
        impl<E> State<E> for $ty {
            type Storage = MapStateStorage<Self, E>;
        }
    };
}

/// Types that can be used as states through `MapStateStorage`.
impl_map_storage!('a, str);
impl_map_storage!(u8);
impl_map_storage!(u32);
impl_map_storage!(u64);
impl_map_storage!(i8);
impl_map_storage!(i32);
impl_map_storage!(i64);
