//! ECS input bundle

use crate::{BindingError, BindingTypes, Bindings, Context, InputSystemDesc};
use amethyst_config::{Config, ConfigError};
use amethyst_core::{
    ecs::prelude::{DispatcherBuilder, World},
    SystemBundle, SystemDesc,
};
use amethyst_error::Error;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error, fmt, path::Path};

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
pub struct InputBundle<C: Context, T: BindingTypes> {
    bindings: Option<HashMap<C, Bindings<T>>>,
    #[cfg(feature = "sdl_controller")]
    controller_mappings: Option<ControllerMappings>,
}

impl<C: Context, T: BindingTypes> InputBundle<C, T> {
    /// Create a new input bundle with no bindings
    pub fn new() -> Self {
        Default::default()
    }

    /// Use the provided bindings with the `InputHandler`
    pub fn with_bindings(mut self, bindings: HashMap<C, Bindings<T>>) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Load bindings from file
    pub fn with_bindings_from_file<P: AsRef<Path>>(
        self,
        file: P,
    ) -> Result<Self, BindingsFileError<T>>
    where
        Bindings<T>: for<'de> Deserialize<'de> + Serialize,
    {
        let mut bindings = match HashMap::<C, Bindings<T>>::load(&file) {
            Ok(bindings) => bindings,
            Err(e) => match Bindings::<T>::load(&file) {
                Ok(_) => return Err(BindingsFileError::OldFormatInUse),
                Err(_) => return Err(e.into()),
            },
        };

        for bindings in bindings.values_mut() {
            bindings.check_invariants()?;
        }
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

impl<'a, 'b, C: Context, T: BindingTypes> SystemBundle<'a, 'b> for InputBundle<C, T> {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        #[cfg(feature = "sdl_controller")]
        {
            use super::SdlEventsSystem;
            builder.add_thread_local(
                // TODO: improve errors when migrating to failure
                SdlEventsSystem::<C, T>::new(world, self.controller_mappings).unwrap(),
            );
        }
        builder.add(
            InputSystemDesc::<C, T>::new(self.bindings).build(world),
            "input_system",
            &[],
        );
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
    /// Old bindings format detected
    OldFormatInUse,
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
            BindingsFileError::OldFormatInUse => write!(
                f,
                "Old input bindings file format detected, for help migrating please see https://book.amethyst.rs/stable/appendices/b_migration_notes/input_context_migration.html"
            ),
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
            BindingsFileError::OldFormatInUse => None,
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
