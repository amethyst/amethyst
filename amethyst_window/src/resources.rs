/// World resource that stores screen dimensions.
#[derive(Debug, PartialEq, Clone)]
pub struct ScreenDimensions {
    /// Screen width in pixels (px).
    pub(crate) w: f64,
    /// Screen height in pixels (px).
    pub(crate) h: f64,
    /// Width divided by height.
    aspect_ratio: f32,
    /// The ratio between the backing framebuffer resolution and the window size in screen pixels.
    /// This is typically one for a normal display and two for a retina display.
    hidpi: f64,
    pub(crate) dirty: bool,
}

impl ScreenDimensions {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new(w: u32, h: u32, hidpi: f64) -> Self {
        ScreenDimensions {
            w: w as f64,
            h: h as f64,
            aspect_ratio: w as f32 / h as f32,
            hidpi,
            dirty: false,
        }
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

    /// Returns the ratio between the backing framebuffer resolution and the window size in screen pixels.
    /// This is typically one for a normal display and two for a retina display.
    pub fn hidpi_factor(&self) -> f64 {
        self.hidpi
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

    /// Updates the hidpi factor stored in this structure.
    ///
    /// Amethyst will call this for you automatically, most engine users won't need this.
    pub fn update_hidpi_factor(&mut self, factor: f64) {
        self.hidpi = factor;
    }
}
