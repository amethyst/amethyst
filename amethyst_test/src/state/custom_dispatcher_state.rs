use std::marker::PhantomData;

use amethyst::{
    core::{
        deferred_dispatcher_operation::{AddSystem, AddSystemDesc, DispatcherOperation},
        SystemDesc,
    },
    ecs::*,
    prelude::*,
};
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
    /// Functions to instantiate state specific dispatcher systems.
    #[derivative(Debug = "ignore")]
    dispatcher_operations: Option<Vec<Box<dyn DispatcherOperation<'a, 'b> + 'a>>>,
    /// State specific dispatcher.
    #[derivative(Debug = "ignore")]
    dispatcher: Option<Dispatcher>,
}

impl<'a, 'b> CustomDispatcherState<'a, 'b> {
    fn new(dispatcher_operations: Vec<Box<dyn DispatcherOperation<'a, 'b> + 'a>>) -> Self {
        CustomDispatcherState {
            dispatcher_operations: Some(dispatcher_operations),
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
            let dispatcher_operations = self.dispatcher_operations.take().expect(
                "Expected `dispatcher_operations` to exist when dispatcher is not yet built.",
            );

            let mut dispatcher_builder = DispatcherBuilder::new();
            dispatcher_operations
                .into_iter()
                .for_each(|dispatcher_operation| {
                    dispatcher_operation
                        .exec(world, &mut dispatcher_builder)
                        .expect("Failed to execute dispatcher operation.");
                });

            let mut dispatcher = dispatcher_builder.build();
            dispatcher.setup(world);
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
        self.dispatcher.as_mut().unwrap().dispatch(&data.world);

        Trans::Pop
    }
}

/// Builder for the `CustomDispatcherState`.
///
/// This allows you to specify which systems you want to run within the state.
#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct CustomDispatcherStateBuilder<'a, 'b> {
    /// Functions to instantiate state specific dispatcher systems.
    #[derivative(Debug = "ignore")]
    #[new(default)]
    dispatcher_operations: Vec<Box<dyn DispatcherOperation<'a, 'b> + 'a>>,
}

impl<'a, 'b: 'a> CustomDispatcherStateBuilder<'a, 'b> {
    /// Registers a `System` with the dispatcher builder.
    ///
    /// # Parameters
    ///
    /// * `system`: Function to instantiate the `System`.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_system<S>(mut self, system: S, name: String, dependencies: Vec<String>) -> Self
    where
        S: for<'c> System<'c> + 'static + Send,
    {
        let dispatcher_operation = Box::new(AddSystem {
            system,
            name,
            dependencies,
        }) as Box<dyn DispatcherOperation<'a, 'b> + 'a>;
        self.dispatcher_operations.push(dispatcher_operation);
        self
    }

    /// Registers a `System` with the dispatcher builder.
    ///
    /// # Parameters
    ///
    /// * `system_desc`: Descriptor to instantiate the `System`.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with_system_desc<SD, S>(
        mut self,
        system_desc: SD,
        name: String,
        dependencies: Vec<String>,
    ) -> Self
    where
        SD: SystemDesc<'a, 'b, S> + 'a,
        S: for<'c> System<'c> + 'a + Send,
    {
        let dispatcher_operation = Box::new(AddSystemDesc {
            system_desc,
            name,
            dependencies,
            marker: PhantomData::<S>,
        }) as Box<dyn DispatcherOperation<'a, 'b> + 'a>;
        self.dispatcher_operations.push(dispatcher_operation);
        self
    }

    /// Builds and returns the `CustomDispatcherState`.
    pub fn build(self) -> CustomDispatcherState<'a, 'b> {
        CustomDispatcherState::new(self.dispatcher_operations)
    }
}
