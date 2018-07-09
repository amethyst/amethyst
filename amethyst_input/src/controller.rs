use event::InputEvent;

/// Controller axes matching SDL controller model
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ControllerAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

/// Controller buttons matching SDL controller model
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ControllerButton {
    A,
    B,
    X,
    Y,
    DPadDown,
    DPadLeft,
    DPadRight,
    DPadUp,
    LeftShoulder,
    RightShoulder,
    LeftStick,
    RightStick,
    Back,
    Start,
    Guide,
}

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ControllerEvent {
    ControllerAxisMoved {
        which: u32,
        axis: ControllerAxis,
        value: f64,
    },
    ControllerButtonPressed {
        which: u32,
        button: ControllerButton,
    },
    ControllerButtonReleased {
        which: u32,
        button: ControllerButton,
    },
    ControllerDisconnected {
        which: u32,
    },
    ControllerConnected {
        which: u32,
    },
}

impl<'a, T> Into<InputEvent<T>> for &'a ControllerEvent {
    fn into(self) -> InputEvent<T> {
        use self::ControllerEvent::*;
        match *self {
            ControllerAxisMoved { which, axis, value } => {
                InputEvent::ControllerAxisMoved { which, axis, value }
            }
            ControllerButtonPressed { which, button } => {
                InputEvent::ControllerButtonPressed { which, button }
            }
            ControllerButtonReleased { which, button } => {
                InputEvent::ControllerButtonReleased { which, button }
            }
            ControllerConnected { which } => InputEvent::ControllerConnected { which },
            ControllerDisconnected { which } => InputEvent::ControllerDisconnected { which },
        }
    }
}
