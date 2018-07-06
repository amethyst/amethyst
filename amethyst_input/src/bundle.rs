//! ECS input bundle

use std::hash::Hash;
use std::path::Path;
use std::result::Result as StdResult;

use amethyst_config::{Config, ConfigError};
use amethyst_core::bundle::{Result, SystemBundle};
use amethyst_core::specs::prelude::DispatcherBuilder;
use serde::de::DeserializeOwned;
use serde::Serialize;

use {Bindings, InputSystem};

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
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct InputBundle<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    bindings: Option<Bindings<AX, AC>>,
}

impl<AX, AC> InputBundle<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    /// Create a new input bundle with no bindings
    pub fn new() -> Self {
        Default::default()
    }

    /// Use the provided bindings with the `InputHandler`
    pub fn with_bindings(mut self, bindings: Bindings<AX, AC>) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Load bindings from file
    pub fn with_bindings_from_file<P: AsRef<Path>>(self, file: P) -> StdResult<Self, ConfigError>
    where
        AX: DeserializeOwned + Serialize,
        AC: DeserializeOwned + Serialize,
    {
        Ok(self.with_bindings(Bindings::load_no_fallback(file)?))
    }
}

impl<'a, 'b, AX, AC> SystemBundle<'a, 'b> for InputBundle<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(
            InputSystem::<AX, AC>::new(self.bindings),
            "input_system",
            &[],
        );
        Ok(())
    }
}
