//! `amethyst` rendering ecs resources

use smallvec::SmallVec;
use winit::Window;

use color::Rgba;

/// The ambient color of a scene
#[derive(Clone, Debug, Default)]
pub struct AmbientColor(pub Rgba);

impl AsRef<Rgba> for AmbientColor {
    fn as_ref(&self) -> &Rgba {
        &self.0
    }
}

/// This specs resource with id 0 permits sending commands to the
/// renderer internal window.
#[derive(Default)]
pub struct WindowMessages {
    // It's unlikely we'll get more than one command per frame
    // 1 Box also makes this the same size as a Vec, so this costs
    // no more space in the structure than a Vec would.
    //
    // NOTE TO FUTURE AUTHORS: This could be an FnOnce but that's not possible
    // right now as of 2017-10-02 because FnOnce isn't object safe.  It might
    // be possible as soon as FnBox stabilizes.  For now I'll use FnMut instead.
    pub(crate) queue: SmallVec<[Box<FnMut(&Window) + Send + Sync + 'static>; 2]>,
}

impl WindowMessages {
    /// Create a new `WindowMessages`
    pub fn new() -> Self {
        Default::default()
    }

    /// Execute this closure on the `winit::Window` next frame.
    pub fn send_command<F>(&mut self, command: F)
    where
        F: FnMut(&Window) + Send + Sync + 'static,
    {
        self.queue.push(Box::new(command));
    }
}

/// World resource that stores screen dimensions.
#[derive(Debug)]
pub struct ScreenDimensions {
    /// Screen width in pixels (px).
    w: f32,
    /// Screen height in pixels (px).
    h: f32,
    /// Width divided by height.
    aspect_ratio: f32,
    pub(crate) dirty: bool,
}

impl ScreenDimensions {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new(w: u32, h: u32) -> ScreenDimensions {
        ScreenDimensions {
            w: w as f32,
            h: h as f32,
            aspect_ratio: w as f32 / h as f32,
            dirty: false,
        }
    }

    /// Returns the current width of the window.
    ///
    /// This is returned as a float for user convenience, as this is typically used with other
    /// float values.  This will only ever be a non-negative integer though.
    pub fn width(&self) -> f32 {
        self.w
    }

    /// Returns the current height of the window.
    ///
    /// This is returned as a float for user convenience, as this is typically used with other
    /// float values.  This will only ever be a non-negative integer though.
    pub fn height(&self) -> f32 {
        self.h
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
    pub fn update(&mut self, w: u32, h: u32) {
        self.w = w as f32;
        self.h = h as f32;
        self.aspect_ratio = w as f32 / h as f32;
        self.dirty = true;
    }
}
