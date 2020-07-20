//! ECS input bundle

use crate::{build_input_system, BindingError, BindingTypes, InputHandler, InputEvent, Bindings};
use amethyst_config::{Config, ConfigError};
use amethyst_core::{
    shrev::EventChannel,
    dispatcher::{DispatcherBuilder, Stage, SystemBundle},
    ecs::prelude::*,
};
use amethyst_error::Error;
use derivative::Derivative;
use std::{error, fmt, path::Path};

#[cfg(feature = "sdl_controller")]
use crate::sdl_events_system::ControllerMappings;

/// Bundle for adding the `InputHandler`.
///
/// This also adds the Winit EventHandler and the `InputEvent<T>` EventHandler
/// where `T::Action` is the type for Actions you have assigned here.
///
/// ## Type parameters
///
/// T: The type used to identify input binding types.
///
/// String is appropriate for either of these if you don't know what to use.
///
/// ## Errors
///
/// No errors returned from this bundle.
///
#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct InputBundle<T: BindingTypes> {
    bindings: Option<Bindings<T>>,
    #[cfg(feature = "sdl_controller")]
    controller_mappings: Option<ControllerMappings>,
}

impl<T: BindingTypes> InputBundle<T> {
    /// Create a new input bundle with no bindings
    pub fn new() -> Self {
        Default::default()
    }

    /// Use the provided bindings with the `InputHandler`
    pub fn with_bindings(mut self, bindings: Bindings<T>) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Load bindings from file
    pub fn with_bindings_from_file<P: AsRef<Path>>(
        self,
        file: P,
    ) -> Result<Self, BindingsFileError<T>>
    where
        Bindings<T>: Config,
    {
        let mut bindings = Bindings::load(file)?;
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
    pub fn with_sdl_controller_mappings_from_file<P: AsRef<Path>>(mut self, file: P) -> Self {
        use std::path::PathBuf;

        let path_buf = PathBuf::from(file.as_ref());
        self.controller_mappings = Some(ControllerMappings::FromPath(path_buf));
        self
    }
}

impl<T: BindingTypes> SystemBundle for InputBundle<T> {
    fn build(
        self,
        world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        #[cfg(feature = "sdl_controller")]
        {
            use super::SdlEventsSystem;
            builder.add_thread_local(
                // TODO: improve errors when migrating to failure
                SdlEventsSystem::<T>::new(world, self.controller_mappings).unwrap(),
            );
        }

        builder.add_system(Stage::Begin, build_input_system::<T>(self.bindings));
        Ok(())
    }
}

/// An error occurred while loading the bindings file.
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub enum BindingsFileError<T: BindingTypes> {
    /// Problem in amethyst_config
    ConfigError(ConfigError),
    /// Problem with the bindings themselves.
    BindingError(BindingError<T>),
}

impl<T: BindingTypes> fmt::Display for BindingsFileError<T>
where
    T::Axis: fmt::Display,
    T::Action: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindingsFileError::ConfigError(..) => write!(f, "Configuration error"),
            BindingsFileError::BindingError(..) => write!(f, "Binding error"),
        }
    }
}

impl<T: BindingTypes> error::Error for BindingsFileError<T>
where
    T::Axis: fmt::Display,
    T::Action: fmt::Display,
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            BindingsFileError::ConfigError(ref e) => Some(e),
            BindingsFileError::BindingError(ref e) => Some(e),
        }
    }
}

impl<T: BindingTypes> From<BindingError<T>> for BindingsFileError<T> {
    fn from(error: BindingError<T>) -> Self {
        BindingsFileError::BindingError(error)
    }
}

impl<T: BindingTypes> From<ConfigError> for BindingsFileError<T> {
    fn from(error: ConfigError) -> Self {
        BindingsFileError::ConfigError(error)
    }
}
