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

/// Bundle for adding the `InputHandler` and input bindings
///
/// ## Errors
///
/// No errors returned from this bundle.
///
pub struct InputBundle<T>
where
    T: Hash + Eq,
{
    bindings: Option<Bindings<T>>,
}

impl<T> InputBundle<T>
where
    T: Hash + Eq + DeserializeOwned + Serialize + Default,
{
    /// Create a new input bundle with no bindings
    pub fn new() -> Self {
        Self { bindings: None }
    }

    /// Use the provided bindings with the `InputHandler`
    pub fn with_bindings(mut self, bindings: Bindings<T>) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Load bindings from file
    pub fn with_bindings_from_file<P: AsRef<Path>>(self, file: P) -> Self {
        self.with_bindings(Bindings::load(file))
    }
}

impl<'a, 'b, T, B> ECSBundle<'a, 'b, T> for InputBundle<B>
where
    B: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        let mut input = InputHandler::new();
        if let Some(ref bindings) = self.bindings {
            input.bindings = bindings.clone();
        }

        let mut winit_handler = EventHandler::<Event>::new();
        let reader_id = winit_handler.register_reader();
        Ok(
            builder
                .with_resource(input)
                .with_resource(winit_handler)
                .with_resource(EventHandler::<InputEvent<B>>::new())
                .with(InputSystem::<B>::new(reader_id), "input_system", &[]),
        )
    }
}
