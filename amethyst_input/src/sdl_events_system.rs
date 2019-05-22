use std::{fmt, marker::PhantomData, path::PathBuf};

use sdl2::{
    self,
    controller::{AddMappingError, Axis, Button, GameController},
    event::Event,
    EventPump, GameControllerSubsystem, Sdl,
};

use amethyst_core::{
    ecs::prelude::{Resources, RunNow, SystemData, Write},
    shrev::EventChannel,
};

use super::{
    controller::{ControllerAxis, ControllerButton, ControllerEvent},
    BindingTypes, InputEvent, InputHandler,
};

/// A collection of errors that can occur in the SDL system.
#[derive(Debug)]
pub enum SdlSystemError {
    /// Failure initializing SDL context
    ContextInit(String),
    /// Failure initializing SDL controller subsystem
    ControllerSubsystemInit(String),
    /// Failure adding a controller mapping
    AddMappingError(AddMappingError),
}

impl fmt::Display for SdlSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SdlSystemError::ContextInit(ref msg) => write!(f, "Failed to initialize SDL: {}", msg),
            SdlSystemError::ControllerSubsystemInit(ref msg) => {
                write!(f, "Failed to initialize SDL controller subsystem: {}", msg)
            }
            SdlSystemError::AddMappingError(ref err) => {
                write!(f, "Failed to load controller mappings: {}", err)
            }
        }
    }
}

/// Different ways to pass in a controller mapping for an SDL controller.
pub enum ControllerMappings {
    /// Provide mappings from a file
    FromPath(PathBuf),
    /// Provide mappings programmatically via a `String`.
    FromString(String),
}

/// A system that pumps SDL events into the `amethyst_input` APIs.
pub struct SdlEventsSystem<T: BindingTypes> {
    #[allow(dead_code)]
    sdl_context: Sdl,
    event_pump: Option<EventPump>,
    controller_subsystem: GameControllerSubsystem,
    /// Vector of opened controllers and their corresponding joystick indices
    opened_controllers: Vec<(u32, GameController)>,
    marker: PhantomData<T>,
}

type SdlEventsData<'a, T> = (
    Write<'a, InputHandler<T>>,
    Write<'a, EventChannel<InputEvent<<T as BindingTypes>::Action>>>,
);

impl<'a, T: BindingTypes> RunNow<'a> for SdlEventsSystem<T> {
    fn run_now(&mut self, res: &'a Resources) {
        let (mut handler, mut output) = SdlEventsData::fetch(res);

        let mut event_pump = self
            .event_pump
            .take()
            .expect("Unreachable: `event_pump` is always reinserted after `take`");
        for event in event_pump.poll_iter() {
            // handle appropriate events locally
            self.handle_sdl_event(&event, &mut handler, &mut output);
        }
        self.event_pump = Some(event_pump);
    }

    fn setup(&mut self, res: &mut Resources) {
        let (mut handler, mut output) = SdlEventsData::fetch(res);
        self.initialize_controllers(&mut handler, &mut output);
    }
}

impl<T: BindingTypes> SdlEventsSystem<T> {
    /// Creates a new instance of this system with the provided controller mappings.
    pub fn new(mappings: Option<ControllerMappings>) -> Result<Self, SdlSystemError> {
        let sdl_context = sdl2::init().map_err(|e| SdlSystemError::ContextInit(e))?;
        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| SdlSystemError::ContextInit(e))?;
        let controller_subsystem = sdl_context
            .game_controller()
            .map_err(|e| SdlSystemError::ControllerSubsystemInit(e))?;

        match mappings {
            Some(ControllerMappings::FromPath(p)) => {
                controller_subsystem
                    .load_mappings(p)
                    .map_err(|e| SdlSystemError::AddMappingError(e))?;
            }
            Some(ControllerMappings::FromString(s)) => {
                controller_subsystem
                    .add_mapping(s.as_str())
                    .map_err(|e| SdlSystemError::AddMappingError(e))?;
            }
            None => {}
        };

        Ok(SdlEventsSystem {
            sdl_context,
            event_pump: Some(event_pump),
            controller_subsystem,
            opened_controllers: vec![],
            marker: PhantomData,
        })
    }

    fn handle_sdl_event(
        &mut self,
        event: &Event,
        handler: &mut InputHandler<T>,
        output: &mut EventChannel<InputEvent<T::Action>>,
    ) {
        use self::ControllerEvent::*;

        match *event {
            Event::ControllerAxisMotion {
                which, axis, value, ..
            } => {
                handler.send_controller_event(
                    &ControllerAxisMoved {
                        which: which as u32,
                        axis: axis.into(),
                        value: if value > 0 {
                            (value as f64) / 32767f64
                        } else {
                            (value as f64) / 32768f64
                        },
                    },
                    output,
                );
            }
            Event::ControllerButtonDown { which, button, .. } => {
                handler.send_controller_event(
                    &ControllerButtonPressed {
                        which: which as u32,
                        button: button.into(),
                    },
                    output,
                );
            }
            Event::ControllerButtonUp { which, button, .. } => {
                handler.send_controller_event(
                    &ControllerButtonReleased {
                        which: which as u32,
                        button: button.into(),
                    },
                    output,
                );
            }
            Event::ControllerDeviceRemoved { which, .. } => {
                self.close_controller(which as u32);
                handler.send_controller_event(
                    &ControllerDisconnected {
                        which: which as u32,
                    },
                    output,
                );
            }
            Event::ControllerDeviceAdded { which, .. } => {
                self.open_controller(which).map(|idx| {
                    handler.send_controller_event(&ControllerConnected { which: idx }, output);
                });
            }
            _ => {}
        }
    }

    fn open_controller(&mut self, which: u32) -> Option<u32> {
        if self.controller_subsystem.is_game_controller(which) {
            self.controller_subsystem.open(which).ok().map(|c| {
                let id = c.instance_id() as u32;
                self.opened_controllers.push((which, c));
                id
            })
        } else {
            None
        }
    }

    fn close_controller(&mut self, which: u32) {
        let index = self
            .opened_controllers
            .iter()
            .position(|(_, c)| c.instance_id() as u32 == which);
        if let Some(i) = index {
            self.opened_controllers.swap_remove(i);
        }
    }

    fn initialize_controllers(
        &mut self,
        handler: &mut InputHandler<T>,
        output: &mut EventChannel<InputEvent<T::Action>>,
    ) {
        use crate::controller::ControllerEvent::ControllerConnected;

        if let Ok(available) = self.controller_subsystem.num_joysticks() {
            for id in 0..available {
                self.open_controller(id).map(|idx| {
                    handler.send_controller_event(&ControllerConnected { which: idx }, output);
                });
            }
        }
    }
}

impl From<Button> for ControllerButton {
    fn from(button: Button) -> Self {
        match button {
            Button::A => ControllerButton::A,
            Button::B => ControllerButton::B,
            Button::X => ControllerButton::X,
            Button::Y => ControllerButton::Y,
            Button::DPadDown => ControllerButton::DPadDown,
            Button::DPadLeft => ControllerButton::DPadLeft,
            Button::DPadRight => ControllerButton::DPadRight,
            Button::DPadUp => ControllerButton::DPadUp,
            Button::LeftShoulder => ControllerButton::LeftShoulder,
            Button::RightShoulder => ControllerButton::RightShoulder,
            Button::LeftStick => ControllerButton::LeftStick,
            Button::RightStick => ControllerButton::RightStick,
            Button::Back => ControllerButton::Back,
            Button::Start => ControllerButton::Start,
            Button::Guide => ControllerButton::Guide,
        }
    }
}

impl From<Axis> for ControllerAxis {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::LeftX => ControllerAxis::LeftX,
            Axis::LeftY => ControllerAxis::LeftY,
            Axis::RightX => ControllerAxis::RightX,
            Axis::RightY => ControllerAxis::RightY,
            Axis::TriggerLeft => ControllerAxis::LeftTrigger,
            Axis::TriggerRight => ControllerAxis::RightTrigger,
        }
    }
}
