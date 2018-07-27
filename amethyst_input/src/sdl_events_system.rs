use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::PathBuf;

use amethyst_core::shrev::EventChannel;
use amethyst_core::specs::prelude::{Resources, RunNow, SystemData, Write};
use sdl2;
use sdl2::controller::{AddMappingError, Axis, Button, GameController};
use sdl2::event::Event;
use sdl2::{EventPump, GameControllerSubsystem, Sdl};

use super::controller::{ControllerAxis, ControllerButton, ControllerEvent};
use super::{InputEvent, InputHandler};

#[derive(Debug)]
pub enum SdlSystemError {
    ContextInit(String),
    ControllerSubsystemInit(String),
    AddMappingError(AddMappingError),
}

impl fmt::Display for SdlSystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

pub enum ControllerMappings {
    FromPath(PathBuf),
    FromString(String),
}

pub struct SdlEventsSystem<AX, AC>
where
    AX: Hash + Eq,
    AC: Hash + Eq,
{
    #[allow(dead_code)]
    sdl_context: Sdl,
    event_pump: Option<EventPump>,
    controller_subsystem: GameControllerSubsystem,
    /// Vector of opened controllers and their corresponding joystick indices
    opened_controllers: Vec<(u32, GameController)>,
    marker: PhantomData<(AX, AC)>,
}

type SdlEventsData<'a, AX, AC> = (
    Write<'a, InputHandler<AX, AC>>,
    Write<'a, EventChannel<InputEvent<AC>>>,
);

impl<'a, AX, AC> RunNow<'a> for SdlEventsSystem<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
    fn run_now(&mut self, res: &'a Resources) {
        let (mut handler, mut output) = SdlEventsData::fetch(res);

        let mut event_pump = self.event_pump.take().unwrap();
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

impl<AX, AC> SdlEventsSystem<AX, AC>
where
    AX: Hash + Eq + Clone + Send + Sync + 'static,
    AC: Hash + Eq + Clone + Send + Sync + 'static,
{
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
        handler: &mut InputHandler<AX, AC>,
        output: &mut EventChannel<InputEvent<AC>>,
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
        let index = self.opened_controllers
            .iter()
            .position(|(_, c)| c.instance_id() as u32 == which);
        if let Some(i) = index {
            self.opened_controllers.swap_remove(i);
        }
    }

    fn initialize_controllers(
        &mut self,
        handler: &mut InputHandler<AX, AC>,
        output: &mut EventChannel<InputEvent<AC>>,
    ) {
        use controller::ControllerEvent::ControllerConnected;

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
