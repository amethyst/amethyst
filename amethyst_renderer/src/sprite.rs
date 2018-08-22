use amethyst_assets::{
    Asset, Error as AssetsError, ErrorKind as AssetsErrorKind, Handle, ProcessingState,
    Result as AssetsResult, SimpleFormat,
};
use amethyst_core::specs::prelude::{Component, VecStorage};
use fnv::FnvHashMap;
use ron::de::from_bytes as from_ron_bytes;

/// An asset handle to sprite sheet metadata.
pub type SpriteSheetHandle = Handle<SpriteSheet>;

/// Meta data for a sprite sheet texture.
///
/// Contains a handle to the texture and the sprite coordinates on the texture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheet {
    /// Index into `MaterialTextureSet` of the texture for this sprite sheet.
    pub texture_id: u64,
    /// A list of sprites in this sprite sheet.
    pub sprites: Vec<Sprite>,
}

impl Asset for SpriteSheet {
    const NAME: &'static str = "renderer::SpriteSheet";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl From<SpriteSheet> for AssetsResult<ProcessingState<SpriteSheet>> {
    fn from(sprite_sheet: SpriteSheet) -> AssetsResult<ProcessingState<SpriteSheet>> {
        Ok(ProcessingState::Loaded(sprite_sheet))
    }
}

/// Dimensions and texture coordinates of each sprite in a sprite sheet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    /// Pixel width of the sprite
    pub width: f32,
    /// Pixel height of the sprite
    pub height: f32,
    /// Number of pixels to shift the sprite to the left and down relative to the entity
    pub offsets: [f32; 2],
    /// Texture coordinates of the sprite
    pub tex_coords: TextureCoordinates,
}

/// Texture coordinates of the sprite
///
/// The coordinates should be normalized to a value between 0.0 and 1.0:
///
/// * X axis: 0.0 is the left side and 1.0 is the right side.
/// * Y axis: 0.0 is the bottom and 1.0 is the top.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureCoordinates {
    /// Normalized left x coordinate
    pub left: f32,
    /// Normalized right x coordinate
    pub right: f32,
    /// Normalized bottom y coordinate
    pub bottom: f32,
    /// Normalized top y coordinate
    pub top: f32,
}

impl From<((f32, f32), [f32; 4])> for Sprite {
    fn from((dimensions, tex_coords): ((f32, f32), [f32; 4])) -> Self {
        Self::from((dimensions, [0.0; 2], tex_coords))
    }
}

impl From<((f32, f32), [f32; 2], [f32; 4])> for Sprite {
    fn from(((width, height), offsets, tex_coords): ((f32, f32), [f32; 2], [f32; 4])) -> Self {
        Sprite {
            width,
            height,
            offsets,
            tex_coords: TextureCoordinates::from(tex_coords),
        }
    }
}

impl From<((f32, f32), (f32, f32))> for TextureCoordinates {
    fn from(((left, right), (bottom, top)): ((f32, f32), (f32, f32))) -> Self {
        TextureCoordinates {
            left,
            right,
            bottom,
            top,
        }
    }
}

impl From<[f32; 4]> for TextureCoordinates {
    fn from(uv: [f32; 4]) -> Self {
        TextureCoordinates {
            left: uv[0],
            right: uv[1],
            bottom: uv[2],
            top: uv[3],
        }
    }
}

/// Information for rendering a sprite.
///
/// Instead of using a `Mesh` on a `DrawFlat` render pass, we can use a simpler set of shaders to
/// render sprites. This struct carries the information necessary for the sprite pass.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteRender {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: SpriteSheetHandle,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
    /// Whether the sprite should be flipped horizontally
    pub flip_horizontal: bool,
    /// Whether the sprite should be flipped vertically
    pub flip_vertical: bool,
}

impl Component for SpriteRender {
    type Storage = VecStorage<Self>;
}

/// Sprite sheets used by sprite render animations
///
/// In sprite animations, it is plausible to switch the `SpriteSheet` during the animation.
/// `Animation`s require their primitives to be `Copy`. However, `Handle<SpriteSheet>`s are `Clone`
/// but not `Copy`. Therefore, to allow switching of the `SpriteSheet`, we use a `Copy` ID, and map
/// that to the sprite sheet handle so that it can be looked up when being sampled in the animation.
#[derive(Debug, Default)]
pub struct SpriteSheetSet {
    sprite_sheets: FnvHashMap<u64, SpriteSheetHandle>,
    sprite_sheet_inverse: FnvHashMap<SpriteSheetHandle, u64>,
}

impl SpriteSheetSet {
    /// Create new sprite sheet set
    pub fn new() -> Self {
        SpriteSheetSet {
            sprite_sheets: FnvHashMap::default(),
            sprite_sheet_inverse: FnvHashMap::default(),
        }
    }

    /// Retrieve the handle for a given index
    pub fn handle(&self, id: u64) -> Option<SpriteSheetHandle> {
        self.sprite_sheets.get(&id).cloned()
    }

    /// Retrieve the index for a given handle
    pub fn id(&self, handle: &SpriteSheetHandle) -> Option<u64> {
        self.sprite_sheet_inverse.get(handle).cloned()
    }

    /// Insert a sprite sheet handle at the given index
    pub fn insert(&mut self, id: u64, handle: SpriteSheetHandle) {
        self.sprite_sheets.insert(id, handle.clone());
        self.sprite_sheet_inverse.insert(handle, id);
    }

    /// Remove the given index
    pub fn remove(&mut self, id: u64) {
        if let Some(handle) = self.sprite_sheets.remove(&id) {
            self.sprite_sheet_inverse.remove(&handle);
        }
    }

    /// Get number of sprite sheets in the set
    pub fn len(&self) -> usize {
        self.sprite_sheets.len()
    }

    /// Returns whether the set contains any sprite sheets
    pub fn is_empty(&self) -> bool {
        self.sprite_sheets.is_empty()
    }

    /// Remove all sprite sheet handles in the set
    pub fn clear(&mut self) {
        self.sprite_sheets.clear();
        self.sprite_sheet_inverse.clear();
    }
}

/// Allows loading of sprite lists in RON format.
#[derive(Clone, Deserialize, Serialize)]
pub struct SpriteSheetFormat;

impl SimpleFormat<SpriteSheet> for SpriteSheetFormat {
    const NAME: &'static str = "SPRITE_SHEET";

    type Options = u64;

    fn import(&self, bytes: Vec<u8>, texture_id: Self::Options) -> AssetsResult<SpriteSheet> {
        let sprites: Vec<Sprite> = from_ron_bytes(&bytes).map_err(|_| {
            AssetsError::from_kind(AssetsErrorKind::Format(
                "Failed to parse sprites Ron file for SpriteSheet",
            ))
        })?;
        Ok(SpriteSheet {
            texture_id,
            sprites,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Sprite, TextureCoordinates};

    #[test]
    fn texture_coordinates_from_tuple_maps_fields_correctly() {
        assert_eq!(
            TextureCoordinates {
                left: 0.,
                right: 0.5,
                bottom: 0.75,
                top: 1.0,
            },
            ((0.0, 0.5), (0.75, 1.0)).into()
        );
    }

    #[test]
    fn texture_coordinates_from_slice_maps_fields_correctly() {
        assert_eq!(
            TextureCoordinates {
                left: 0.,
                right: 0.5,
                bottom: 0.75,
                top: 1.0,
            },
            [0.0, 0.5, 0.75, 1.0].into()
        );
    }

    #[test]
    fn sprite_from_tuple_maps_fields_correctly() {
        assert_eq!(
            Sprite {
                width: 10.,
                height: 40.,
                offsets: [5., 20.],
                tex_coords: TextureCoordinates {
                    left: 0.,
                    right: 0.5,
                    bottom: 0.75,
                    top: 1.0,
                },
            },
            ((10., 40.), [5., 20.], [0.0, 0.5, 0.75, 1.0]).into()
        );
    }

    #[test]
    fn sprite_offsets_default_to_zero() {
        assert_eq!(
            Sprite {
                width: 10.,
                height: 40.,
                offsets: [0., 0.],
                tex_coords: TextureCoordinates {
                    left: 0.,
                    right: 0.5,
                    bottom: 0.75,
                    top: 1.0,
                },
            },
            ((10., 40.), [0.0, 0.5, 0.75, 1.0]).into()
        );
    }
}
