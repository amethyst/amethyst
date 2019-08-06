use std::marker::PhantomData;

use amethyst::{core::SystemDesc, ecs::prelude::*, prelude::*};

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
    system_fns: Option<Vec<Box<dyn SystemFn<'a, 'b> + 'a>>>,
    /// State specific dispatcher.
    #[derivative(Debug = "ignore")]
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> CustomDispatcherState<'a, 'b> {
    fn new(system_fns: Vec<Box<dyn SystemFn<'a, 'b> + 'a>>) -> Self {
        CustomDispatcherState {
            system_fns: Some(system_fns),
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
            let system_fns = self
                .system_fns
                .take()
                .expect("Expected `system_fns` to exist when dispatcher is not yet built.");

            let mut dispatcher_builder = DispatcherBuilder::new();
            system_fns.into_iter().for_each(|system_fn| {
                system_fn.build(world, &mut dispatcher_builder);
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
    system_fns: Vec<Box<dyn SystemFn<'a, 'b> + 'a>>,
}

impl<'a, 'b: 'a> CustomDispatcherStateBuilder<'a, 'b> {
    /// Registers a `System` with the dispatcher builder.
    ///
    /// # Parameters
    ///
    /// * `system_desc`: Descriptor to instantiate the `System`.
    /// * `name`: Name to register the system with, used for dependency ordering.
    /// * `deps`: Names of systems that must run before this system.
    pub fn with<SD, S>(mut self, system_desc: SD, name: String, deps: Vec<String>) -> Self
    where
        SD: SystemDesc<'a, 'b, S> + 'a,
        S: for<'c> System<'c> + Send + Sync + 'b,
    {
        let system_fn_data = SystemFnData::new(system_desc, name, deps);
        self.system_fns.push(Box::new(system_fn_data));
        self
    }

    /// Builds and returns the `CustomDispatcherState`.
    pub fn build(self) -> CustomDispatcherState<'a, 'b> {
        CustomDispatcherState::new(self.system_fns)
    }
}

trait SystemFn<'a, 'b> {
    fn build(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    );
}

/// Sized type to wrap functions that create `System`s.
#[derive(Debug, new)]
struct SystemFnData<'a, 'b, SD, S>
where
    SD: SystemDesc<'a, 'b, S>,
    S: for<'s> System<'s> + Send,
{
    /// Descriptor to instantiate `System` to add to the dispatcher.
    system_desc: SD,
    /// Name to register the system with.
    system_name: String,
    /// Names of the system dependencies.
    system_dependencies: Vec<String>,
    /// Marker.
    system_marker: PhantomData<(&'a SD, &'b S)>,
}

impl<'a, 'b, SD, S> SystemFn<'a, 'b> for SystemFnData<'a, 'b, SD, S>
where
    SD: SystemDesc<'a, 'b, S>,
    S: for<'s> System<'s> + Send + 'a,
{
    fn build(
        self: Box<Self>,
        world: &mut World,
        dispatcher_builder: &mut DispatcherBuilder<'a, 'b>,
    ) {
        let system = self.system_desc.build(world);
        dispatcher_builder.add(
            system,
            &self.system_name,
            &self
                .system_dependencies
                .iter()
                .map(|dep| dep.as_ref())
                .collect::<Vec<&str>>(),
        )
    }
}
