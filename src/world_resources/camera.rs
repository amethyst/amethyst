//! This module contains `Projection` and `Camera` structs,
//! which are used in `GfxDevice::render_world` to construct
//! projection and view transformations.

/// A projection enum which is required to create a `Camera`.
#[derive(Copy, Clone)]
pub enum Projection {
    Perspective {
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

/// A `Camera` world resource, it is added by default to `ecs::World` by `Application`.
/// Projection and view matricies are constructed from it and used by `GfxDevice::render_world` method.
#[derive(Copy, Clone)]
pub struct Camera {
    pub projection: Projection,
    /// Point at which camera is located.
    pub eye: [f32; 3],
    /// Position of camera target (point at which camera is pointed).
    pub target: [f32; 3],
    /// Vector defining the up direction for the camera.
    pub up: [f32; 3],
}

impl Camera {
    /// Create a new `Camera` from `Projection` enum and eye, target, up vectors
    pub fn new(projection: Projection, eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Camera {
        Camera {
            projection: projection,
            eye: eye,
            target: target,
            up: up,
        }
    }
}
