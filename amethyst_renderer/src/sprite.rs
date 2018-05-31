use amethyst_assets::{Asset, Handle};
use amethyst_core::specs::prelude::VecStorage;

/// An asset handle to sprite sheet metadata.
pub type SpriteSheetHandle = Handle<SpriteSheet>;

/// Meta data for a sprite sheet texture.
///
/// Contains a handle to the texture and the sprite coordinates on the texture.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteSheet {
    /// Index into `MaterialTextureSet` of the texture for this sprite sheet.
    pub index: usize,
    /// A list of sprites in this sprite sheet.
    pub sprites: Vec<Sprite>,
}

impl Asset for SpriteSheet {
    const NAME: &'static str = "renderer::Sprite";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

/// A description of a frame in a sprite sheet.
///
/// These should be in normalized coordinates:
///
/// * X axis: 0.0 is the left side and 1.0 is the right side.
/// * Y axis: 0.0 is the top and 1.0 is the bottom.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    /// Normalized left x coordinate
    pub left: f32,
    /// Normalized right x coordinate
    pub right: f32,
    /// Normalized top y coordinate
    pub top: f32,
    /// Normalized bottom y coordinate
    pub bottom: f32,
}

impl From<((f32, f32), (f32, f32))> for Sprite {
    fn from(((left, right), (top, bottom)): ((f32, f32), (f32, f32))) -> Self {
        Sprite {
            left,
            right,
            top,
            bottom,
        }
    }
}

impl From<[f32; 4]> for Sprite {
    fn from(uv: [f32; 4]) -> Self {
        Sprite {
            left: uv[0],
            right: uv[1],
            top: uv[2],
            bottom: uv[3],
        }
    }
}

#[cfg(test)]
mod test {
    use super::Sprite;

    #[test]
    fn sprite_from_tuple_maps_coordinates_correctly() {
        assert_eq!(
            Sprite {
                left: 0.,
                right: 0.5,
                top: 0.75,
                bottom: 1.0,
            },
            ((0.0, 0.5), (0.75, 1.0)).into()
        );
    }

    #[test]
    fn sprite_from_slice_maps_coordinates_correctly() {
        assert_eq!(
            Sprite {
                left: 0.,
                right: 0.5,
                top: 0.75,
                bottom: 1.0,
            },
            [0.0, 0.5, 0.75, 1.0].into()
        );
    }
}
