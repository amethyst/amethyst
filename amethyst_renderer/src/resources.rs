//! `amethyst` rendering ecs resources

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::{Entity, Write};
use color::Rgba;
use smallvec::SmallVec;
use vertex::PosColorNorm;
use winit::Window;

/// The ambient color of a scene
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AmbientColor(pub Rgba);

impl AsRef<Rgba> for AmbientColor {
    fn as_ref(&self) -> &Rgba {
        &self.0
    }
}

impl<'a> PrefabData<'a> for AmbientColor {
    type SystemData = Write<'a, AmbientColor>;
    type Result = ();

    fn load_prefab(
        &self,
        _: Entity,
        ambient: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        ambient.0 = self.0.clone();
        Ok(())
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
    /// The ratio between the backing framebuffer resolution and the window size in screen pixels.
    /// This is typically one for a normal display and two for a retina display.
    hidpi: f32,
    pub(crate) dirty: bool,
}

impl ScreenDimensions {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new(w: u32, h: u32, hidpi: f32) -> ScreenDimensions {
        ScreenDimensions {
            w: w as f32,
            h: h as f32,
            aspect_ratio: w as f32 / h as f32,
            hidpi,
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

    /// Returns the ratio between the backing framebuffer resolution and the window size in screen pixels.
    /// This is typically one for a normal display and two for a retina display.
    pub fn hidpi_factor(&self) -> f32 {
        self.hidpi
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

/// Resource that stores debug lines to be rendered in DebugLinesPass draw pass
#[derive(Debug)]
pub struct DebugLines {
    /// Lines to be rendered this frame
    pub lines: Vec<PosColorNorm>,
}

impl DebugLines {
    /// Creates a new screen dimensions object with the given width and height.
    pub fn new() -> DebugLines {
        DebugLines {
            lines: Vec::<PosColorNorm>::new(),
        }
    }

    /// Builder method to pre-allocate a number of line.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.lines = Vec::<PosColorNorm>::with_capacity(capacity);
        self
    }

    /// Adds a line to be rendered by giving a position and a direction.
    pub fn add_as_direction(&mut self, position: [f32; 3], direction: [f32; 3], color: Rgba) {
        let vertex = PosColorNorm {
            position: position,
            color: color.into(),
            normal: direction,
        };

        self.lines.push(vertex);
    }

    /// Adds a line to be rendered by giving a start and an end position.
    pub fn add_as_line(&mut self, start: [f32; 3], end: [f32; 3], color: Rgba) {
        let vertex = PosColorNorm {
            position: start,
            color: color.into(),
            normal: [end[0] - start[0], end[1] - start[1], end[2] - start[2]],
        };

        self.lines.push(vertex);
    }
}
