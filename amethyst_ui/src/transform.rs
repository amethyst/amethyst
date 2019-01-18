use std::marker::PhantomData;

use amethyst_core::specs::prelude::{
    Component, DenseVecStorage, Entities, Entity, FlaggedStorage, Join, ReadStorage,
};

use serde::{Deserialize, Serialize};
use shred_derive::SystemData;

use super::{Anchor, ScaleMode, Stretch};

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
/// Values are in pixel and the position is calculated from the bottom left of the screen
/// to the center of the ui element's area.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiTransform {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// Indicates where the element sits, relative to the parent (or to the screen, if there is no parent)
    pub anchor: Anchor,
    /// If a child ui element needs to fill its parent this can be used to stretch it to the appropriate size.
    pub stretch: Stretch,
    /// X coordinate, 0 is the left edge of the screen. If scale_mode is set to pixel then the width of the
    /// screen in pixel is the right edge.  If scale_mode is percent then the right edge is 1.
    ///
    /// Centered in the middle of the ui element.
    pub local_x: f32,
    /// Y coordinate, 0 is the bottom edge of the screen. If scale_mode is set to pixel then the height of the
    /// screen in pixel is the top edge.  If scale_mode is percent then the top edge is 1.
    ///
    /// Centered in the middle of the ui element.
    pub local_y: f32,
    /// Z order, entities with a higher Z order will be rendered on top of entities with a lower
    /// Z order.
    pub local_z: f32,
    /// The width of this UI element.
    pub width: f32,
    /// The height of this UI element.
    pub height: f32,
    /// Global x position set by the `UiTransformSystem`.
    pub(crate) pixel_x: f32,
    /// Global y position set by the `UiTransformSystem`.
    pub(crate) pixel_y: f32,
    /// Global z position set by the `UiTransformSystem`.
    pub(crate) global_z: f32,
    /// Width in pixels, used for rendering.  Duplicate of `width` if `scale_mode == ScaleMode::Pixel`.
    pub(crate) pixel_width: f32,
    /// Height in pixels, used for rendering.  Duplicate of `height` if `scale_mode == ScaleMode::Pixel`.
    pub(crate) pixel_height: f32,
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
        anchor: Anchor,
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
    ) -> UiTransform {
        UiTransform {
            id,
            anchor,
            stretch: Stretch::NoStretch,
            local_x: x,
            local_y: y,
            local_z: z,
            width,
            height,
            pixel_x: x,
            pixel_y: y,
            global_z: z,
            pixel_width: width,
            pixel_height: height,
            scale_mode: ScaleMode::Pixel,
            opaque: true,
            pd: PhantomData,
        }
    }
    /// Checks if the input position is in the UiTransform rectangle.
    /// Uses local coordinates (ignores layouting).
    pub fn position_inside_local(&self, x: f32, y: f32) -> bool {
        x > self.local_x - self.width / 2.0
            && y > self.local_y - self.height / 2.0
            && x < self.local_x + self.width / 2.0
            && y < self.local_y + self.height / 2.0
    }

    /// Checks if the input position is in the UiTransform rectangle.
    pub fn position_inside(&self, x: f32, y: f32) -> bool {
        x > self.pixel_x - self.pixel_width / 2.0
            && y > self.pixel_y - self.pixel_height / 2.0
            && x < self.pixel_x + self.pixel_width / 2.0
            && y < self.pixel_y + self.pixel_height / 2.0
    }

    /// Renders this UI element by evaluating transform as a percentage of the parent size,
    /// rather than rendering it with pixel units.
    pub fn as_percent(mut self) -> Self {
        self.scale_mode = ScaleMode::Percent;
        self
    }

    /// Sets the opaque variable to false, allowing ui events to go through this ui element.
    pub fn as_transparent(mut self) -> Self {
        self.opaque = false;
        self
    }

    /// Adds stretching to this ui element so it can fill its parent.
    pub fn with_stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = stretch;
        self
    }

    /// Returns the global x coordinate of this UiTransform as computed by the `UiTransformSystem`.
    pub fn pixel_x(&self) -> f32 {
        self.pixel_x
    }

    /// Returns the global y coordinate of this UiTransform as computed by the `UiTransformSystem`.
    pub fn pixel_y(&self) -> f32 {
        self.pixel_y
    }

    /// Returns the global z order of this UiTransform as computed by the `UiTransformSystem`.
    pub fn global_z(&self) -> f32 {
        self.global_z
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
        let tr = UiTransform::new("".to_string(), Anchor::TopLeft, 0.0, 0.0, 0.0, 1.0, 1.0);
        let pos = (-0.49, 0.20);
        assert!(tr.position_inside_local(pos.0, pos.1));
        let pos = (-1.49, 1.20);
        assert!(!tr.position_inside_local(pos.0, pos.1));
    }

    #[test]
    fn inside_global() {
        let tr = UiTransform::new("".to_string(), Anchor::TopLeft, 0.0, 0.0, 0.0, 1.0, 1.0);
        let pos = (-0.49, 0.20);
        assert!(tr.position_inside(pos.0, pos.1));
        let pos = (-1.49, 1.20);
        assert!(!tr.position_inside(pos.0, pos.1));
    }
}
