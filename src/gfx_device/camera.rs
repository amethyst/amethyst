/// A projection enum which is required to create a `Camera` component.
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

/// A `Camera` component.
/// If this `Camera` is active then all changes in this component's fields
/// will be applied to the camera that is being used to render the scene.
#[derive(Copy, Clone)]
pub struct Camera {
    pub projection: Projection,
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
}

impl Camera {
    /// Create a new `Camera` component from all the parameters
    /// for projection and view transformations.
    pub fn new(projection: Projection, eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Camera {
        Camera {
            projection: projection,
            eye: eye,
            target: target,
            up: up,
        }
    }
}
