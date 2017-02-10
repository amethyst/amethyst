//! World resource for an orthographic or perspective projection camera.

/// Represents the graphical projection of a `Camera`.
#[derive(Copy, Clone)]
pub enum Projection {
    /// A realistic [perspective projection][pp].
    ///
    /// [pp]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    Perspective {
        /// Field of view, measured in degrees.
        fov: f32,
        /// Aspect ratio of the viewport.
        aspect_ratio: f32,
        /// Distance of the near clipping plane.
        near: f32,
        /// Distance of the far clipping plane.
        far: f32,
    },
    /// An [orthographic projection][op].
    ///
    /// [op]: https://en.wikipedia.org/wiki/Orthographic_projection
    Orthographic {
        /// Distance of the left clipping plane.
        left: f32,
        /// Distance of the right clipping plane.
        right: f32,
        /// Distance of the bottom clipping plane.
        bottom: f32,
        /// Distance of the top clipping plane.
        top: f32,
        /// Distance of the near clipping plane.
        near: f32,
        /// Distance of the far clipping plane.
        far: f32,
    },
}

/// Represents a camera looking around inside a game world.
#[derive(Copy, Clone)]
pub struct Camera {
    /// Graphical projection of the camera.
    pub proj: Projection,
    /// Location of the camera in three-dimensional space.
    pub eye: [f32; 3],
    /// The point at which the camera is looking directly at.
    pub target: [f32; 3],
    /// Upward elevation vector of the camera.
    pub up: [f32; 3],
}

impl Camera {
    /// Creates a new camera with the projection `proj` and the given eye,
    /// target, and up vectors.
    pub fn new(proj: Projection, eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Camera {
        Camera {
            proj: proj,
            eye: eye,
            target: target,
            up: up,
        }
    }
}
