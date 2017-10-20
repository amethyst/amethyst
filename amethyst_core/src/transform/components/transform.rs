//! Global transform component.

use std::borrow::Borrow;

use specs::{Component, FlaggedStorage, VecStorage};

/// Performs a global transformation on the entity (transform from origin).
///
/// Used for rendering position and orientation.
///
/// If this component is used, and `TransformSystem` is not used, then make sure to clear the flags
/// on the `FlaggedStorage` at the appropriate times (before updating any `Transform` in the frame).
/// See documentation on `FlaggedStorage` for more information.
#[derive(Debug, Copy, Clone)]
pub struct Transform(pub [[f32; 4]; 4]);

impl Component for Transform {
    type Storage = FlaggedStorage<Transform, VecStorage<Transform>>;
}

impl Default for Transform {
    fn default() -> Self {
        Transform([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

impl From<[[f32; 4]; 4]> for Transform {
    fn from(matrix: [[f32; 4]; 4]) -> Self {
        Transform(matrix)
    }
}

impl Into<[[f32; 4]; 4]> for Transform {
    fn into(self) -> [[f32; 4]; 4] {
        self.0
    }
}

impl AsRef<[[f32; 4]; 4]> for Transform {
    fn as_ref(&self) -> &[[f32; 4]; 4] {
        &self.0
    }
}


impl Borrow<[[f32; 4]; 4]> for Transform {
    fn borrow(&self) -> &[[f32; 4]; 4] {
        &self.0
    }
}
