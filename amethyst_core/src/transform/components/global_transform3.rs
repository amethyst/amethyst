//! Global transform component.

use std::borrow::Borrow;

use nalgebra::{self as na, Matrix4, Real};
use specs::prelude::{Component, DenseVecStorage, FlaggedStorage};

/// Performs a global transformation on the entity (transform from origin).
///
/// Used for rendering position and orientation.
///
/// If this component is used, and `TransformSystem` is not used, then make sure to clear the flags
/// on the `FlaggedStorage` at the appropriate times (before updating any `Transform` in the frame).
/// See documentation on `FlaggedStorage` for more information.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GlobalTransform3<N: Real>(pub Matrix4<N>);

impl<N: Real> GlobalTransform3<N> {
    /// Checks whether each `f32` of the `GlobalTransform` is finite (not NaN or inf).
    pub fn is_finite(&self) -> bool {
        self.0.as_slice().iter().all(|f| N::is_finite(f))
    }
}

impl<N: Real> Component for GlobalTransform3<N> {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl<N: Real> Default for GlobalTransform3<N> {
    fn default() -> Self {
        GlobalTransform3(na::one())
    }
}

impl<N: Real> GlobalTransform3<N> {
    /// Creates a new `GlobalTransform` in the form of an identity matrix.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<N: Real> From<[[N; 4]; 4]> for GlobalTransform3<N> {
    fn from(matrix: [[N; 4]; 4]) -> Self {
        GlobalTransform3(matrix.into())
    }
}

impl<N: Real> Into<[[N; 4]; 4]> for GlobalTransform3<N> {
    fn into(self) -> [[N; 4]; 4] {
        self.0.into()
    }
}

impl<N: Real> AsRef<[[N; 4]; 4]> for GlobalTransform3<N> {
    fn as_ref(&self) -> &[[N; 4]; 4] {
        self.0.as_ref()
    }
}

impl<N: Real> Borrow<[[N; 4]; 4]> for GlobalTransform3<N> {
    fn borrow(&self) -> &[[N; 4]; 4] {
        self.0.as_ref()
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
