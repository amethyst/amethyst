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
            forward: [1.0, 0.0, 0.0].into(),
            right: [0.0, -1.0, 0.0].into(),
            up: [0.0, 0.0, 1.0].into(),
        }
    }
}
