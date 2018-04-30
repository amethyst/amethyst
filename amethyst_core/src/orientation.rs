//! Orientation of objects

use cgmath::{Matrix3, Vector3};

/// This struct contains 3 unit vectors pointing in the given directions.
///
/// This information relies on the coordinate system in use, otherwise some of the vectors may have
/// incorrect sign.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Orientation {
    /// Forward vector [x, y, z]
    pub forward: Vector3<f32>,
    /// Right vector [x, y, z]
    pub right: Vector3<f32>,
    /// Up vector [x, y, z]
    pub up: Vector3<f32>,
}

impl From<Matrix3<f32>> for Orientation {
    /// Performs the conversion.
    ///
    /// Reverses the z axis matching the GL coordinate system.
    fn from(mat: Matrix3<f32>) -> Self {
        Orientation {
            forward: -mat.z,
            right: mat.x,
            up: mat.y,
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        // Signs depend on coordinate system
        Self {
            forward: -Vector3::unit_z(),
            right: Vector3::unit_x(),
            up: Vector3::unit_y(),
        }
    }
}
