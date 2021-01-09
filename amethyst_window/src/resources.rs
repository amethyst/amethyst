use amethyst_core::math::Vector2;

/// World resource that stores screen dimensions.
#[derive(Debug, PartialEq, Clone)]
pub struct ScreenDimensions {
    /// Screen width in physical pixels (px).
    pub(crate) w: f64,
    /// Screen height in physical pixels (px).
    pub(crate) h: f64,
    /// Width divided by height.
    aspect_ratio: f32,
    pub(crate) dirty: bool,
}

impl ScreenDimensions {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new(w: u32, h: u32) -> Self {
        ScreenDimensions {
            w: f64::from(w),
            h: f64::from(h),
            aspect_ratio: w as f32 / h as f32,
            dirty: false,
        }
    }

    /// Returns the current logical size of window as diagonal vector.
    pub fn diagonal(&self) -> Vector2<f32> {
        Vector2::new(self.width(), self.height())
    }

    /// Returns the current logical width of the window.
    pub fn width(&self) -> f32 {
        self.w as f32
    }

    /// Returns the current logical height of the window.
    pub fn height(&self) -> f32 {
        self.h as f32
    }

    /// Returns the current aspect ratio of the window.
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Updates the width and height of the screen and recomputes the aspect
    /// ratio.
    ///
    /// Only use this if you need to programmatically set the resolution of your game.
    /// This resource is updated automatically by the engine when a resize occurs so you don't need
    /// this unless you want to resize the game window.
    pub fn update(&mut self, w: f64, h: f64) {
        self.w = w;
        self.h = h;
        self.aspect_ratio = w as f32 / h as f32;
        self.dirty = true;
    }
}
