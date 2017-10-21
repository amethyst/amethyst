use specs::{Component, DenseVecStorage};

/// The raw pixels on screen that are populated.
///
/// TODO: Eventually this should be either replaced by a citrine type, or citrine may just
/// populate it.
pub struct UiTransform {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    dirty: bool,
}

impl UiTransform {
    /// Initialize a new UITransform
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> UiTransform {
        Self {
            x,
            y,
            width,
            height,
            dirty: true,
        }
    }

    /// Get the x coordinate
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Get the y coordinate
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Get the width
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get the height
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Get a mutable reference to the x coordinate.
    pub fn x_mut(&mut self) -> &mut f32 {
        self.dirty = true;
        &mut self.x
    }

    /// Get a mutable reference to the y coordinate
    pub fn y_mut(&mut self) -> &mut f32 {
        self.dirty = true;
        &mut self.y
    }

    /// Get a mutable reference to the width
    pub fn width_mut(&mut self) -> &mut f32 {
        self.dirty = true;
        &mut self.width
    }

    /// Get a mutable reference to the height
    pub fn height_mut(&mut self) -> &mut f32 {
        self.dirty = true;
        &mut self.height
    }
}

impl Component for UiTransform {
    type Storage = DenseVecStorage<Self>;
}
