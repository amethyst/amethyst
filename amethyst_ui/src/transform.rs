use amethyst_assets::prefab::{legion_prefab, register_component_type, serde_diff, SerdeDiff};
use amethyst_core::{ecs::*, transform::Parent};
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

use super::{Anchor, ScaleMode, Stretch};

/// Utility lookup for finding UI entities based on `UiTransform` id
#[derive(Debug)]
pub struct UiFinder;

impl UiFinder {
    /// Find the `UiTransform` entity with the given id
    pub fn find<W: EntityStore>(world: &mut W, id: &str) -> Option<Entity> {
        <(Entity, &UiTransform)>::query()
            .iter(world)
            .filter(|(_, transform)| transform.id == id)
            .map(|(e, _)| *e)
            .next()
    }
}

/// The UiTransform represents the transformation of a ui element.
/// Values are in pixel and the position is calculated from the bottom left of the screen
/// to the center of the ui element's area.
#[derive(Clone, Default, Debug, Serialize, Deserialize, TypeUuid, SerdeDiff)]
#[serde(default)]
#[uuid = "d900c11d-f8b2-4145-8a19-537a67d5ee85"]
pub struct UiTransform {
    /// An identifier. Serves no purpose other than to help you distinguish between UI elements.
    pub id: String,
    /// Indicates where the element sits, relative to the parent (or to the screen, if there is no parent)
    pub anchor: Anchor,
    /// Indicates where the element sits, relative to itself
    pub pivot: Anchor,
    /// If a child ui element needs to fill its parent this can be used to stretch it to the appropriate size.
    pub stretch: Stretch,
    /// X coordinate, 0 is the left edge of the screen. If scale_mode is set to pixel then the width of the
    /// screen in pixel is the right edge.  If scale_mode is percent then the right edge is 1.
    ///
    /// Centered in the middle of the ui element.
    #[serde(alias = "x")]
    pub local_x: f32,
    /// Y coordinate, 0 is the bottom edge of the screen. If scale_mode is set to pixel then the height of the
    /// screen in pixel is the top edge.  If scale_mode is percent then the top edge is 1.
    ///
    /// Centered in the middle of the ui element.
    #[serde(alias = "y")]
    pub local_y: f32,
    /// Z order, entities with a higher Z order will be rendered on top of entities with a lower
    /// Z order.
    #[serde(alias = "z")]
    pub local_z: f32,
    /// The width of this UI element.
    pub width: f32,
    /// The height of this UI element.
    pub height: f32,
    /// Global x position set by the `UiTransformSystem`.
    #[doc(hidden)]
    #[serde(alias = "x")]
    pub pixel_x: f32,
    /// Global y position set by the `UiTransformSystem`.
    #[doc(hidden)]
    #[serde(alias = "y")]
    pub pixel_y: f32,
    /// Global z position set by the `UiTransformSystem`.
    #[doc(hidden)]
    #[serde(alias = "z")]
    pub global_z: f32,
    /// Width in pixels, used for rendering.  Duplicate of `width` if `scale_mode == ScaleMode::Pixel`.
    #[doc(hidden)]
    #[serde(alias = "width")]
    pub pixel_width: f32,
    /// Height in pixels, used for rendering.  Duplicate of `height` if `scale_mode == ScaleMode::Pixel`.
    #[doc(hidden)]
    #[serde(alias = "height")]
    pub pixel_height: f32,
    /// The scale mode indicates if the position is in pixel or is relative (%) (WIP!) to the parent's size.
    pub scale_mode: ScaleMode,
    /// Indicates if actions on the ui can go through this element.
    /// If set to false, the element will behaves as if it was transparent and will let events go to
    /// the next element (for example, the text on a button).
    pub opaque: bool,
    /// Allows transparent (opaque = false) transforms to still be targeted by the events that pass
    /// through them.
    pub transparent_target: bool,
}

register_component_type!(UiTransform);

impl UiTransform {
    /// Creates a new UiTransform.
    /// By default, it is considered opaque.
    pub fn new(
        id: String,
        anchor: Anchor,
        pivot: Anchor,
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
    ) -> UiTransform {
        UiTransform {
            id,
            anchor,
            pivot,
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
            transparent_target: false,
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
    pub fn into_percent(mut self) -> Self {
        self.scale_mode = ScaleMode::Percent;
        self
    }

    /// Sets the opaque variable to false, allowing ui events to go through this ui element.
    pub fn into_transparent(mut self) -> Self {
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

    /// Returns the width of this UiTransform (in pixels) as computed by the `UiTransformSystem`.
    pub fn pixel_width(&self) -> f32 {
        self.pixel_width
    }

    /// Returns the height of this UiTransform (in pixels) as computed by the `UiTransformSystem`.
    pub fn pixel_height(&self) -> f32 {
        self.pixel_height
    }
}

/// Get the (width, height) in pixels of the parent of this `UiTransform`.
pub fn get_parent_pixel_size<'a, I>(
    maybe_parent: Option<&Parent>,
    mut maybe_transforms: I,
    screen_dimensions: &ScreenDimensions,
) -> (f32, f32)
where
    I: Iterator<Item = (&'a Entity, Option<&'a UiTransform>)>,
{
    if let Some(parent) = maybe_parent {
        let maybe_transform = maybe_transforms.find(|(e, _)| *e == &parent.0);
        if let Some((_, Some(t))) = maybe_transform {
            return (t.pixel_width(), t.pixel_height());
        }
    }
    (screen_dimensions.width(), screen_dimensions.height())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inside_local() {
        let tr = UiTransform::new(
            "".to_string(),
            Anchor::TopLeft,
            Anchor::Middle,
            0.0,
            0.0,
            0.0,
            1.0,
            1.0,
        );
        let pos = (-0.49, 0.20);
        assert!(tr.position_inside_local(pos.0, pos.1));
        let pos = (-1.49, 1.20);
        assert!(!tr.position_inside_local(pos.0, pos.1));
    }

    #[test]
    fn inside_global() {
        let tr = UiTransform::new(
            "".to_string(),
            Anchor::TopLeft,
            Anchor::Middle,
            0.0,
            0.0,
            0.0,
            1.0,
            1.0,
        );
        let pos = (-0.49, 0.20);
        assert!(tr.position_inside(pos.0, pos.1));
        let pos = (-1.49, 1.20);
        assert!(!tr.position_inside(pos.0, pos.1));
    }
}
