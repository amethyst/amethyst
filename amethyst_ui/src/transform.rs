use specs::{Component, DenseVecStorage};


/// The raw pixels on screen that are populated.
///
/// TODO: Eventually this should be either replaced by a citrine type, or citrine may just
/// populate it.
pub struct UiTransform {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// X coordinate, 0 is the left edge, while the width of the screen is the right edge.
    pub x: f32,
    /// Y coordinate, 0 is the top edge, while the height of the screen is the bottom edge.
    pub y: f32,
    /// Z order, entities with a lower Z order will be rendered on top of entities with a higher
    /// Z order.
    pub z: f32,
    /// The width of this UI element
    pub width: f32,
    /// The height of this UI element
    pub height: f32,
}

impl Component for UiTransform {
    type Storage = DenseVecStorage<Self>;
}
