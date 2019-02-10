use serde::{Deserialize, Serialize};

use super::{Button, ControllerAxis};

/// Represents any input represented by a float value from -1 to 1.
/// Retrieve the value of this with [axis_value](struct.InputHandler.html#method.axis_value).
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Axis {
    /// Represents an emulated analogue axis made up of pair of digital inputs,
    /// like W and S keyboard buttons or `DPadUp` and `DPadDown` controller buttons.
    Emulated {
        /// Positive button, when pressed down axis value will return 1 if `neg` is not pressed down.
        pos: Button,
        /// Negative button, when pressed down axis value will return -1 if `pos` is not pressed down.
        neg: Button,
    },
    /// Represents an analogue axis of a controller.
    Controller {
        /// A number representing specific controller, assigned and reused in order of connection.
        controller_id: u32,
        /// The axis being bound
        axis: ControllerAxis,
        /// Whether or not to multiply the axis value by -1.
        invert: bool,
        /// Treat input values from -dead_zone to dead_zone as 0,
        /// linearly interpolate remaining ranges.
        dead_zone: f64,
    },
}
