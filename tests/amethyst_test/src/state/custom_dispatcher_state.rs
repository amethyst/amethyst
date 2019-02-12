use amethyst::{ecs::prelude::*, prelude::*};

use derivative::Derivative;
use derive_new::new;

use crate::GameUpdate;

/// State with a custom dispatcher.
///
/// This allows you to specify which systems you want to run within the state. This should be
/// constructed using the `CustomDispatcherStateBuilder`.
#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct CustomDispatcherState<'a, 'b> {
    /// State specific dispatcher builder.
    #[derivative(Debug = "ignore")]
    dispatcher_builder: Option<DispatcherBuilder<'a, 'b>>,
    /// State specific dispatcher.
    #[derivative(Debug = "ignore")]
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> CustomDispatcherState<'a, 'b> {
    fn new(dispatcher_builder: DispatcherBuilder<'a, 'b>) -> Self {
        CustomDispatcherState {
            dispatcher_builder: Some(dispatcher_builder),
            dispatcher: None,
        }
    }

    /// Sets up the dispatcher for this state.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to operate on.
    fn initialize_dispatcher(&mut self, world: &mut World) {
        if self.dispatcher.is_none() {
            let mut dispatcher = self
                .dispatcher_builder
                .take()
                .expect(
                    "Expected `dispatcher_builder` to exist when `dispatcher` is not yet built.",
                )
                .build();
            dispatcher.setup(&mut world.res);
            self.dispatcher = Some(dispatcher);
        }
    }

    /// Terminates the dispatcher.
    fn terminate_dispatcher(&mut self) {
        self.dispatcher = None;
    }
}

impl<'a, 'b, T, E> State<T, E> for CustomDispatcherState<'a, 'b>
where
    T: GameUpdate,
    E: Send + Sync + 'static,
{
    fn on_start(&mut self, mut data: StateData<'_, T>) {
        self.initialize_dispatcher(&mut data.world);
    }

    fn on_stop(&mut self, _data: StateData<'_, T>) {
        self.terminate_dispatcher();
    }

    fn update(&mut self, data: StateData<'_, T>) -> Trans<T, E> {
        data.data.update(&data.world);
        self.dispatcher.as_mut().unwrap().dispatch(&data.world.res);

        Trans::Pop
    }
}

/// Builder for the `CustomDispatcherState`.
///
/// This allows you to specify which systems you want to run within the state.
#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct CustomDispatcherStateBuilder<'a, 'b> {
    /// State specific dispatcher.
    #[derivative(Debug = "ignore")]
    #[new(value = "DispatcherBuilder::new()")]
    dispatcher_builder: DispatcherBuilder<'a, 'b>,
}

impl<'a, 'b> CustomDispatcherStateBuilder<'a, 'b> {
    /// Registers a `System` with the dispatcher builder.
    ///
    /// # Parameters
    ///
    /// * `system`: `System` to register.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with<Sys>(mut self, system: Sys, name: &str, deps: &[&str]) -> Self
    where
        Sys: for<'c> System<'c> + Send + 'a,
    {
        self.dispatcher_builder.add(system, name, deps);
        self
    }

    /// Builds and returns the `CustomDispatcherState`.
    pub fn build(self) -> CustomDispatcherState<'a, 'b> {
        CustomDispatcherState::new(self.dispatcher_builder)
    }
}
