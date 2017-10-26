//! Orientation of objects

use cgmath::Vector3;

/// Orientation struct.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Orientation {
    /// Forward vector [x, y, z]
    pub forward: Vector3<f32>,
    /// Right vector [x, y, z]
    pub right: Vector3<f32>,
    /// Up vector [x, y, z]
    pub up: Vector3<f32>,
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            forward: Vector3::unit_x(),
            right: -Vector3::unit_y(),
            up: Vector3::unit_z(),
        }
    }
}
