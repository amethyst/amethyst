/// A two dimensional axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Axis2 {
    // The X axis. Often the horizontal (left-right) position.
    X,
    // The Y axis. Often the vertical height.
    Y,
}

/// A three dimensional axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Axis3 {
    // The X axis. Often the horizontal (left-right) position.
    X,
    // The Y axis. Often the vertical height.
    Y,
    // The Z axis. Often the depth.
    Z,
}