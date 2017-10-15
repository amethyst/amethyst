//! ECS input bundle

use std::hash::Hash;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use winit::Event;
use core::bundle::{ECSBundle, Result};

use config::Config;
use ecs::{World, DispatcherBuilder};
use ecs::input::{Bindings, InputEvent, InputHandler, InputSystem};
use shrev::EventChannel;

/// Bundle for adding the `InputHandler`.
///
/// This also adds the Winit EventHandler and the InputEvent<AC> EventHandler
/// where AC is the type for Actions you have assigned here.
///
/// ## Type parameters
///
/// AX: The type used to identify input axes.
/// AC: The type used to identify input actions.
///
/// String is appropriate for either of these if you don't know what to use.
///
/// ## Errors
///
/// No errors returned from this bundle.
///
pub struct InputBundle<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    bindings: Option<Bindings<AX, AC>>,
}

impl<AX, AC> InputBundle<AX, AC>
where
    AX: Hash + Eq + DeserializeOwned + Serialize + Default,
    AC: Hash + Eq + DeserializeOwned + Serialize + Default,
{
    /// Create a new input bundle with no bindings
    pub fn new() -> Self {
        Self { bindings: None }
    }

    /// Use the provided bindings with the `InputHandler`
    pub fn with_bindings(mut self, bindings: Bindings<AX, AC>) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Load bindings from file
    pub fn with_bindings_from_file<P: AsRef<Path>>(self, file: P) -> Self {
        self.with_bindings(Bindings::load(file))
    }
}

impl<'a, 'b, AX, AC> ECSBundle<'a, 'b> for InputBundle<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn build(
        self,
        world: &mut World,
        builder: DispatcherBuilder<'a, 'b>,
    ) -> Result<DispatcherBuilder<'a, 'b>> {
        let mut input = InputHandler::new();
        if let Some(bindings) = self.bindings {
            input.bindings = bindings;
        }

        let reader_id = world
            .read_resource::<EventChannel<Event>>()
            .register_reader();

        world.add_resource(input);
        world.add_resource(EventChannel::<InputEvent<AC>>::with_capacity(2000));
        Ok(
            builder
                .add(InputSystem::<AX, AC>::new(reader_id), "input_system", &[]),
        )
    }
}
