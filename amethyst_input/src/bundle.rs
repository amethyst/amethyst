//! ECS input bundle

use std::{error, fmt, path::Path};

use amethyst_config::{Config, ConfigError};
use amethyst_core::{ecs::*, shrev::EventChannel};
use amethyst_error::Error;
use derivative::Derivative;
use winit::event::Event;

#[cfg(feature = "sdl_controller")]
use crate::sdl_events_system::ControllerMappings;
#[cfg(feature = "sdl_controller")]
use crate::InputEvent;
use crate::{BindingError, Bindings, InputHandler, InputSystem};

/// Bundle for adding the `InputHandler`.
///
/// This also adds the Winit EventHandler and the `InputEvent` EventHandler
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
#[derive(Debug, Default)]
pub struct InputBundle {
    bindings: Option<Bindings>,
    #[cfg(feature = "sdl_controller")]
    controller_mappings: Option<ControllerMappings>,
}

impl InputBundle {
    /// Create a new input bundle with no bindings
    pub fn new() -> Self {
        Default::default()
    }

    /// Use the provided bindings with the `InputHandler`
    pub fn with_bindings(mut self, bindings: Bindings) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Load bindings from file
    pub fn with_bindings_from_file<P: AsRef<Path>>(self, file: P) -> Result<Self, BindingsFileError>
    where
        Bindings: Config,
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

impl SystemBundle for InputBundle {
    fn load(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        let reader = resources
            .get_mut::<EventChannel<Event<'_, ()>>>()
            .expect("Window event channel not found in resources")
            .register_reader();

        let mut handler = InputHandler::new();
        if let Some(bindings) = self.bindings.as_ref() {
            handler.bindings = bindings.clone();
        }

        #[cfg(feature = "sdl_controller")]
        {
            use super::SdlEventsSystem;
            builder.add_thread_local(
                // TODO: improve errors when migrating to failure
                Box::new(
                    SdlEventsSystem::new(
                        &mut handler,
                        &mut resources.get_mut::<EventChannel<InputEvent>>().unwrap(),
                        &self.controller_mappings,
                    )
                    .unwrap(),
                ),
            );
        }

        resources.insert(handler);

        builder.add_system(InputSystem { reader });

        Ok(())
    }
}

/// An error occurred while loading the bindings file.
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub enum BindingsFileError {
    /// Problem in amethyst_config
    ConfigError(ConfigError),
    /// Problem with the bindings themselves.
    BindingError(BindingError),
}

impl fmt::Display for BindingsFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindingsFileError::ConfigError(..) => write!(f, "Configuration error"),
            BindingsFileError::BindingError(..) => write!(f, "Binding error"),
        }
    }
}

impl error::Error for BindingsFileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            BindingsFileError::ConfigError(ref e) => Some(e),
            BindingsFileError::BindingError(ref e) => Some(e),
        }
    }
}

impl From<BindingError> for BindingsFileError {
    fn from(error: BindingError) -> Self {
        BindingsFileError::BindingError(error)
    }
}

impl From<ConfigError> for BindingsFileError {
    fn from(error: ConfigError) -> Self {
        BindingsFileError::ConfigError(error)
    }
}
