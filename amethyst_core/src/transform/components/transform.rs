//! Global transform component.

use std::borrow::Borrow;

use cgmath::{Matrix4, One};
use specs::prelude::{Component, DenseVecStorage, FlaggedStorage};

/// Performs a global transformation on the entity (transform from origin).
///
/// Used for rendering position and orientation.
///
/// If this component is used, and `TransformSystem` is not used, then make sure to clear the flags
/// on the `FlaggedStorage` at the appropriate times (before updating any `Transform` in the frame).
/// See documentation on `FlaggedStorage` for more information.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GlobalTransform(pub Matrix4<f32>);

impl GlobalTransform {
    /// Checks whether each `f32` of the `GlobalTransform` is finite (not NaN or inf).
    pub fn is_finite(&self) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if !self.0[i][j].is_finite() {
                    return false;
                }
            }
        }

        true
    }
}

impl Component for GlobalTransform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl Default for GlobalTransform {
    fn default() -> Self {
        GlobalTransform(Matrix4::one())
    }
}

impl GlobalTransform {
    /// Creates a new `GlobalTransform` in the form of an identity matrix.
    pub fn new() -> Self {
        Default::default()
    }
}

impl From<[[f32; 4]; 4]> for GlobalTransform {
    fn from(matrix: [[f32; 4]; 4]) -> Self {
        GlobalTransform(matrix.into())
    }
}

impl Into<[[f32; 4]; 4]> for GlobalTransform {
    fn into(self) -> [[f32; 4]; 4] {
        self.0.into()
    }
}

impl AsRef<[[f32; 4]; 4]> for GlobalTransform {
    fn as_ref(&self) -> &[[f32; 4]; 4] {
        self.0.as_ref()
    }
}

impl Borrow<[[f32; 4]; 4]> for GlobalTransform {
    fn borrow(&self) -> &[[f32; 4]; 4] {
        self.0.as_ref()
    }
}
