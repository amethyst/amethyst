//! World resource that stores screen dimensions.

/// Abstract representation of a screen.
pub struct ScreenDimensions {
    /// Screen width in pixels (px).
    pub w: f32,
    /// Screen height in pixels (px).
    pub h: f32,
    /// Width divided by height.
    pub aspect_ratio: f32,
}

impl ScreenDimensions {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new(w: u32, h: u32) -> ScreenDimensions {
        ScreenDimensions {
            w: w as f32,
            h: h as f32,
            aspect_ratio: w as f32 / h as f32,
        }
    }

    /// Updates the width and height of the screen and recomputes the aspect
    /// ratio.
    pub fn update(&mut self, w: u32, h: u32) {
        self.w = w as f32;
        self.h = h as f32;
        self.aspect_ratio = w as f32 / h as f32;
    }
}
