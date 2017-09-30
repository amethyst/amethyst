//! ECS input bundle

use std::hash::Hash;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use winit::Event;

use app::ApplicationBuilder;
use config::Config;
use ecs::ECSBundle;
use ecs::input::{Bindings, InputEvent, InputHandler, InputSystem};
use error::Result;
use shrev::EventHandler;

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

impl<'a, 'b, T, AX, AC> ECSBundle<'a, 'b, T> for InputBundle<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        let mut input = InputHandler::new();
        if let Some(ref bindings) = self.bindings {
            input.bindings = bindings.clone();
        }

        let winit_handler = EventHandler::<Event>::new();
        let reader_id = winit_handler.register_reader();
        Ok(
            builder
                .with_resource(input)
                .with_resource(winit_handler)
                .with_resource(EventHandler::<InputEvent<AC>>::new())
                .with(InputSystem::<AX, AC>::new(reader_id), "input_system", &[]),
        )
    }
}
