use std::marker::PhantomData;

use amethyst_core::specs::prelude::{Component, DenseVecStorage, Entities, Entity, FlaggedStorage,
                                    Join, ReadStorage};

use ScaleMode;

/// Utility `SystemData` for finding UI entities based on `UiTransform` id
#[derive(SystemData)]
pub struct UiFinder<'a> {
    entities: Entities<'a>,
    storage: ReadStorage<'a, UiTransform>,
}

impl<'a> UiFinder<'a> {
    /// Find the `UiTransform` entity with the given id
    pub fn find(&self, id: &str) -> Option<Entity> {
        (&*self.entities, &self.storage)
            .join()
            .find(|(_, transform)| transform.id == id)
            .map(|(entity, _)| entity)
    }
}

/// The UiTransform represents the transformation of a ui element.
/// Values are in pixel and the position is calculated from the top left of the screen
/// to the center of the ui element's area.
#[derive(Clone, Debug)]
pub struct UiTransform {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// X coordinate, 0 is the left edge of the screen, while the width of the screen in pixel is the right edge.
    /// Centered in the middle of the ui element.
    pub local_x: f32,
    /// Y coordinate, 0 is the top edge of the screen, while the height of the screen in pixel is the bottom edge.
    /// Centered in the middle of the ui element.
    pub local_y: f32,
    /// Z order, entities with a lower Z order will be rendered on top of entities with a higher
    /// Z order.
    pub local_z: f32,
    /// The width of this UI element in pixel.
    pub width: f32,
    /// The height of this UI element in pixel.
    pub height: f32,
    /// The UI element tab order.  When the player presses tab the UI focus will shift to the
    /// UI element with the next highest tab order, or if another element with the same tab_order
    /// as this one exists they are ordered according to Entity creation order.  Shift-tab walks
    /// this ordering backwards.
    pub tab_order: i32,
    /// Global x position set by the `UiParentSystem` and `UiLayoutSystem` systems.
    pub global_x: f32,
    /// Global y position set by the `UiParentSystem` and `UiLayoutSystem` systems.
    pub global_y: f32,
    /// Global z position set by the `UiParentSystem` and `UiLayoutSystem` systems.
    pub global_z: f32,
    /// The scale mode indicates if the position is in pixel or is relative (%) (WIP!) to the parent's size.
    pub scale_mode: ScaleMode,
    /// Indicates if actions on the ui can go through this element.
    /// If set to false, the element will behaves as if it was transparent and will let events go to
    /// the next element (for example, the text on a button).
    pub opaque: bool,
    /// A private field to keep this from being initialized without new.
    pd: PhantomData<u8>,
}

impl UiTransform {
    /// Creates a new UiTransform.
    /// By default, it is considered opaque.
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
            local_x: x,
            local_y: y,
            local_z: z,
            width,
            height,
            tab_order,
            global_x: x,
            global_y: y,
            global_z: z,
            scale_mode: ScaleMode::Pixel,
            opaque: true,
            pd: PhantomData,
        }
    }
    /// Checks if the input position is in the UiTransform rectangle.
    /// Uses local coordinates (ignores layouting).
    pub fn position_inside_local(&self, x: f32, y: f32) -> bool {
        x > self.local_x - self.width / 2.0 && y > self.local_y - self.height / 2.0
            && x < self.local_x + self.width / 2.0 && y < self.local_y + self.height / 2.0
    }

    /// Checks if the input position is in the UiTransform rectangle.
    pub fn position_inside(&self, x: f32, y: f32) -> bool {
        x > self.global_x - self.width / 2.0 && y > self.global_y - self.height / 2.0
            && x < self.global_x + self.width / 2.0 && y < self.global_y + self.height / 2.0
    }

    /// Currently unused. Will be implemented in a future PR.
    pub fn as_percent(mut self) -> Self {
        self.scale_mode = ScaleMode::Percent;
        self
    }

    /// Sets the opaque variable to false, allowing ui events to go through this ui element.
    pub fn as_transparent(mut self) -> Self {
        self.opaque = false;
        self
    }
}

impl Component for UiTransform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inside_local() {
        let tr = UiTransform::new("".to_string(), 0.0, 0.0, 0.0, 1.0, 1.0, 0);
        let pos = (-0.49, 0.20);
        assert!(tr.position_inside_local(pos.0, pos.1));
        let pos = (-1.49, 1.20);
        assert!(!tr.position_inside_local(pos.0, pos.1));
    }

    #[test]
    fn inside_global() {
        let tr = UiTransform::new("".to_string(), 0.0, 0.0, 0.0, 1.0, 1.0, 0);
        let pos = (-0.49, 0.20);
        assert!(tr.position_inside(pos.0, pos.1));
        let pos = (-1.49, 1.20);
        assert!(!tr.position_inside(pos.0, pos.1));
    }
}
