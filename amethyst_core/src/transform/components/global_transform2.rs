//! Global transform component.

use std::borrow::Borrow;

use nalgebra::{self as na, Matrix3, Vector2, Real};
use specs::prelude::{Component, DenseVecStorage, FlaggedStorage};

/// Performs a global transformation on the entity (transform from origin).
///
/// Used for rendering position and orientation.
///
/// If this component is used, and `TransformSystem` is not used, then make sure to clear the flags
/// on the `FlaggedStorage` at the appropriate times (before updating any `Transform` in the frame).
/// See documentation on `FlaggedStorage` for more information.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GlobalTransform2<N: Real> {
    /// The position, rotation and scale matrix
    pub matrix: Matrix3<N>,
    
    pub dimensions: Vector2<N>,
    pub layer: i32,
}

impl<N: Real> GlobalTransform2<N> {
    /// Checks whether each `f32` of the `GlobalTransform` is finite (not NaN or inf).
    pub fn is_finite(&self) -> bool {
        self.matrix.as_slice().iter().all(|f| N::is_finite(f))
    }
}

impl<N: Real> Component for GlobalTransform2<N> {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl<N: Real> Default for GlobalTransform2<N> {
    fn default() -> Self {
        GlobalTransform2{
            matrix: na::one(),
            dimensions: na::zero(),
            layer: 0,
        }
    }
}

impl<N: Real> GlobalTransform2<N> {
    /// Creates a new `GlobalTransform` in the form of an identity matrix.
    pub fn new() -> Self {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::GlobalTransform;

    #[test]
    fn is_finite() {
        let mut transform = GlobalTransform::default();
        assert!(transform.is_finite());

        transform.0.fill_row(2, std::f32::NAN);
        assert!(!transform.is_finite());
    }
}
