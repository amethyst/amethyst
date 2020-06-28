use serde::{Deserialize, Serialize};

use super::{Button, ControllerAxis, MouseAxis};

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
        /// A number representing a specific controller, assigned and reused in order of connection.
        controller_id: u32,
        /// The axis being bound
        axis: ControllerAxis,
        /// Whether or not to multiply the axis value by -1.
        invert: bool,
        /// Treat input values from -dead_zone to dead_zone as 0,
        /// linearly interpolate remaining ranges.
        dead_zone: f64,
    },
    /// Represents a mouse as a 2D input device
    Mouse {
        /// The axis being bound
        axis: MouseAxis,
        /// Should the API be allowed to return values outside [-1..1]?
        over_extendable: bool,
        /// Zone to which the movement is relative
        radius: f32,
    },
    /// Represents the wheel on a PC mouse.
    MouseWheel {
        /// If this value is true then this axis is for the horizontal mouse wheel rather than the vertical mouse wheel.
        ///
        /// You almost always want this false.
        horizontal: bool,
    },
    /// Represents multiple input alternatives. Allows to bind more than one input to a single axis.
    Multiple(Vec<Axis>),
}

pub(super) enum Conflict {
    Button,
    ControllerAxis,
    MouseAxis,
    MouseWheelAxis,
}

impl Axis {
    pub(super) fn conflicts_with_button(&self, other: &Button) -> bool {
        match self {
            Axis::Emulated { pos, neg } => other == pos || other == neg,
            Axis::Multiple(axes) => axes.iter().any(|a| a.conflicts_with_button(other)),
            _ => false,
        }
    }

    pub(super) fn conflicts_with_axis(&self, other: &Axis) -> Option<Conflict> {
        if let Axis::Multiple(axes) = other {
            if let Some(inner_conflict) = axes
                .iter()
                .map(|a| self.conflicts_with_axis(a))
                .find(|x| x.is_some())
            {
                return inner_conflict;
            }
        }

        match self {
            Axis::Emulated {
                pos: ref self_pos,
                neg: ref self_neg,
            } => {
                if let Axis::Emulated { pos, neg } = other {
                    if self_pos == pos || self_pos == neg || self_neg == pos || self_neg == neg {
                        return Some(Conflict::Button);
                    }
                }
            }
            Axis::Controller {
                controller_id: ref self_controller_id,
                axis: ref self_axis,
                ..
            } => {
                if let Axis::Controller {
                    controller_id,
                    axis,
                    ..
                } = other
                {
                    if self_controller_id == controller_id && self_axis == axis {
                        return Some(Conflict::ControllerAxis);
                    }
                }
            }
            Axis::Mouse {
                axis: self_axis, ..
            } => {
                if let Axis::Mouse { axis, .. } = other {
                    if self_axis == axis {
                        return Some(Conflict::MouseAxis);
                    }
                }
            }
            Axis::MouseWheel {
                horizontal: self_horizontal,
            } => {
                if let Axis::MouseWheel { horizontal } = other {
                    if self_horizontal == horizontal {
                        return Some(Conflict::MouseWheelAxis);
                    }
                }
            }
            Axis::Multiple(axes) => {
                if let Some(inner_conflict) = axes
                    .iter()
                    .map(|a| a.conflicts_with_axis(other))
                    .find(|x| x.is_some())
                {
                    return inner_conflict;
                }
            }
        }
        None
    }
}
