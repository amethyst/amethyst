use super::ScaleMode;
use specs::{Component, DenseVecStorage, FlaggedStorage};
use std::marker::PhantomData;

/// The raw pixels on screen that are populated.
///
/// TODO: Eventually this should be either replaced by a citrine type, or citrine may just
/// populate it.
#[derive(Clone, Debug)]
pub struct UiTransform {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// X coordinate, 0 is the left edge, while the width of the screen is the right edge.
    /// Centered in the middle of the ui element.
    pub x: f32,
    /// Y coordinate, 0 is the top edge, while the height of the screen is the bottom edge.
    /// Centered in the middle of the ui element.
    pub y: f32,
    /// Z order, entities with a lower Z order will be rendered on top of entities with a higher
    /// Z order.
    pub z: f32,
    /// The width of this UI element
    pub width: f32,
    /// The height of this UI element
    pub height: f32,
    /// The UI element tab order.  When the player presses tab the UI focus will shift to the
    /// UI element with the next highest tab order, or if another element with the same tab_order
    /// as this one exists they are ordered according to Entity creation order.  Shift-tab walks
    /// this ordering backwards.
    pub tab_order: i32,
    /// Calculated x position by the `UiParentSystem` and `UiLayoutSystem`.
    pub calculated_x: f32,
    /// Calculated y position by the `UiParentSystem` and `UiLayoutSystem`.
    pub calculated_y: f32,
    /// Calculated z position by the `UiParentSystem` and `UiLayoutSystem`.
    pub calculated_z: f32,
    /// WIP
    /// The scale mode indicates if the position is in pixel or is relative (%) to the parent's size.
    pub scale_mode: ScaleMode,
    /// A private field to keep this from being initialized without new.
    pd: PhantomData<u8>,
}

impl UiTransform {
    /// Creates a new UiTransform
    pub fn new(
        id: String,
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
        tab_order: i32,
    ) -> UiTransform {
        UiTransform {
            id,
            x,
            y,
            z,
            width,
            height,
            tab_order,
            calculated_x: x,
            calculated_y: y,
            calculated_z: z,
            scale_mode: ScaleMode::Pixel,
            pd: PhantomData,
        }
    }
    /// Checks if the input position is in the UiTransform rectangle.
    pub fn position_inside(&self, x: f32, y: f32) -> bool {
        x > self.x - self.width / 2.0 && y > self.y - self.height / 2.0
            && x < self.x + self.width / 2.0 && y < self.y + self.height / 2.0
    }

    /// Currently unused. Will be implemented in a future PR.
    pub fn as_percent(mut self) -> Self {
        self.scale_mode = ScaleMode::Percent;
        self
    }
}

impl Component for UiTransform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
