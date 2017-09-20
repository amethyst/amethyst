//! ECS input bundle

use std::path::Path;

use app::ApplicationBuilder;
use config::Config;
use ecs::ECSBundle;
use ecs::input::{Bindings, InputHandler};
use error::Result;

/// Bundle for adding the `InputHandler` and input bindings
///
/// ## Errors
///
/// No errors returned from this bundle.
///
pub struct InputBundle {
    bindings: Option<Bindings>,
}

impl InputBundle {
    /// Create a new input bundle with no bindings
    pub fn new() -> Self {
        Self { bindings: None }
    }

    /// Use the provided bindings with the `InputHandler`
    pub fn with_bindings(mut self, bindings: Bindings) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Load bindings from file
    pub fn with_bindings_from_file<P: AsRef<Path>>(self, file: P) -> Self {
        self.with_bindings(Bindings::load(file))
    }
}

impl<'a, 'b, T> ECSBundle<'a, 'b, T> for InputBundle {
    fn build(
        &self,
        builder: ApplicationBuilder<'a, 'b, T>,
    ) -> Result<ApplicationBuilder<'a, 'b, T>> {
        let mut input = InputHandler::new();
        if let Some(ref bindings) = self.bindings {
            input.bindings = bindings.to_owned();
        }
        Ok(builder.with_resource(input))
    }
}
