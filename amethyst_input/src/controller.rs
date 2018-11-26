use crate::event::InputEvent;

/// Controller axes matching SDL controller model
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ControllerAxis {
    /// The X axis on the left stick
    LeftX,
    /// The Y axis on the left stick
    LeftY,
    /// The X axis on the right stick
    RightX,
    /// The Y axis on the right stick
    RightY,
    /// The analog left trigger, not to be confused with the left bumper.
    LeftTrigger,
    /// The analog right trigger, not to be confused with the right bumper.
    RightTrigger,
}

/// Controller buttons matching SDL controller model
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ControllerButton {
    /// The A button, typically the lower button in the "diamond" of buttons on the right side
    /// of the controller.
    A,
    /// The B button, typically the right button in the "diamond" of buttons on the right side
    /// of the controller.
    B,
    /// The X button, typically the left button in the "diamond" of buttons on the right side
    /// of the controller.
    X,
    /// The Y button, typically the top button in the "diamond" of buttons on the right side
    /// of the controller.
    Y,
    /// The dpad button pointed towards the player
    DPadDown,
    /// The dpad button pointed to the player's left
    DPadLeft,
    /// The dpad button pointed to the player's right
    DPadRight,
    /// The dpad button pointed away from the player.
    DPadUp,
    /// The digital left shoulder bumper. Usually located above the left trigger.
    LeftShoulder,
    /// The digital right shoulder bumper. Usually located above the right trigger.
    RightShoulder,
    /// If your press the left analog stick into the controller this button is pressed.
    LeftStick,
    /// If your press the right analog stick into the controller this button is pressed.
    RightStick,
    /// The back button is typically a button slightly left of center with a leftward arrow on it.
    Back,
    /// The start button is typically a button slightly right of center with a rightward arrow on it.
    Start,
    /// The centermost button on the controller. Large and green on an Xbox controller.
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
