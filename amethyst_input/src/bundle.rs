//! ECS input bundle

use derivative::Derivative;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    convert::From,
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::Hash,
    path::Path,
    result::Result as StdResult,
};

use amethyst_config::{Config, ConfigError};
use amethyst_core::{
    bundle::{Result, SystemBundle},
    specs::prelude::DispatcherBuilder,
};

use crate::{BindingError, Bindings, InputSystem};

#[cfg(feature = "sdl_controller")]
use crate::sdl_events_system::ControllerMappings;

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
    AX: Hash + Eq + Clone,
    AC: Hash + Eq + Clone,
{
    bindings: Option<Bindings<AX, AC>>,
    #[cfg(feature = "sdl_controller")]
    controller_mappings: Option<ControllerMappings>,
}

impl<AX, AC> InputBundle<AX, AC>
where
    AX: Hash + Eq + Clone,
    AC: Hash + Eq + Clone,
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
    pub fn with_bindings_from_file<P: AsRef<Path>>(
        self,
        file: P,
    ) -> StdResult<Self, BindingsFileError<AX, AC>>
    where
        AX: DeserializeOwned + Serialize + Display,
        AC: DeserializeOwned + Serialize + Display,
    {
        let mut bindings = Bindings::load_no_fallback(file)?;
        bindings.check_invariants()?;
        Ok(self.with_bindings(bindings))
    }

    /// Load SDL controller mappings from file
    #[cfg(feature = "sdl_controller")]
    pub fn with_sdl_controller_mappings(mut self, mappings: String) -> Self {
        self.controller_mappings = Some(ControllerMappings::FromString(mappings));
        self
    }

    /// Load SDL controller mappings from file
    #[cfg(feature = "sdl_controller")]
    pub fn with_sdl_controller_mappings_from_file<P: AsRef<Path>>(mut self, file: P) -> Self
    where
        AX: DeserializeOwned + Serialize,
        AC: DeserializeOwned + Serialize,
    {
        use std::path::PathBuf;

        let path_buf = PathBuf::from(file.as_ref());
        self.controller_mappings = Some(ControllerMappings::FromPath(path_buf));
        self
    }
}

impl<'a, 'b, AX, AC> SystemBundle<'a, 'b> for InputBundle<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        #[cfg(feature = "sdl_controller")]
        {
            use super::SdlEventsSystem;
            builder.add_thread_local(
                // TODO: improve errors when migrating to failure
                SdlEventsSystem::<AX, AC>::new(self.controller_mappings).unwrap(),
            );
        }
        builder.add(
            InputSystem::<AX, AC>::new(self.bindings),
            "input_system",
            &[],
        );
        Ok(())
    }
}

/// An error occurred while loading the bindings file.
#[derive(Debug)]
pub enum BindingsFileError<AX, AC> {
    /// Problem in amethyst_config
    ConfigError(ConfigError),
    /// Problem with the bindings themselves.
    BindingError(BindingError<AX, AC>),
}

impl<AX, AC> Display for BindingsFileError<AX, AC>
where
    AX: Display,
    AC: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BindingsFileError::ConfigError(ref err) => Display::fmt(err, f),
            BindingsFileError::BindingError(ref err) => Display::fmt(err, f),
        }
    }
}

impl<AX, AC> Error for BindingsFileError<AX, AC>
where
    AX: Debug + Display,
    AC: Debug + Display,
{
}

impl<AX, AC> From<BindingError<AX, AC>> for BindingsFileError<AX, AC> {
    fn from(error: BindingError<AX, AC>) -> Self {
        BindingsFileError::BindingError(error)
    }
}

impl<AX, AC> From<ConfigError> for BindingsFileError<AX, AC> {
    fn from(error: ConfigError) -> Self {
        BindingsFileError::ConfigError(error)
    }
}
