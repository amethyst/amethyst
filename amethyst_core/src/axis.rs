use serde::{Deserialize, Serialize};

/// A two dimensional axis.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Axis2 {
    /// The X axis. Often the horizontal (left-right) position.
    X,
    /// The Y axis. Often the vertical height.
    Y,
}

/// A three dimensional axis.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Axis3 {
    /// The X axis. Often the horizontal (left-right) position.
    X,
    /// The Y axis. Often the vertical height.
    Y,
    /// The Z axis. Often the depth.
    Z,
}
