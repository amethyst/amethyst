//! World resource for passing clear_color and clear_depth parameters to GfxDevice::render_world.

/// Resource holding clear_color and clear_depth values.
pub struct ClearColor {
    /// Clear color value, main color target is
    /// cleared with this color value every frame.
    pub clear_color: [f32; 4],
    /// Clear depth value, main depth target is
    /// cleared with this depth value every frame.
    pub clear_depth: f32,
}
