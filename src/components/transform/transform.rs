use ecs::{Component, VecStorage};

/// Absolute transformation (transformed from origin).
/// Used for rendering position and orientation.
/// Every `Renderable`, `Transform` pair attached to an `Entity`
/// inside the `World` is rendered by `GfxDevice::render_world` method.
#[derive(Debug, Copy, Clone)]
pub struct Transform(pub [[f32; 4]; 4]);

impl Component for Transform {
    type Storage = VecStorage<Transform>;
}

impl Default for Transform {
    fn default() -> Self {
        Transform([[1.0, 0.0, 0.0, 0.0],
                   [0.0, 1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]])
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
