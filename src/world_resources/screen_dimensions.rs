//! This module contains `ScreenDimensions` struct, which holds width, height, and aspect ratio of the screen.

/// `ScreenDimensions` is added to `ecs::World` as a resource by default.
/// It's fields are set to the correct values every frame.
pub struct ScreenDimensions {
    /// Width of screen.
    pub w: f32,
    /// Height of screen.
    pub h: f32,
    /// Width divided by height.
    pub aspect_ratio: f32,
}

impl ScreenDimensions {
    pub fn new(w: u32, h: u32) -> ScreenDimensions {
        ScreenDimensions {
            w: w as f32,
            h: h as f32,
            aspect_ratio: w as f32 / h as f32,
        }
    }

    pub fn update(&mut self, w: u32, h: u32) {
        self.w = w as f32;
        self.h = h as f32;
        self.aspect_ratio = w as f32 / h as f32;
    }
}
