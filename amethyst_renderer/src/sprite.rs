use ron::de::from_bytes as from_ron_bytes;
use serde::{Deserialize, Serialize};

use amethyst_assets::{
    Asset, Error as AssetsError, ErrorKind as AssetsErrorKind, Handle, ProcessingState,
    Result as AssetsResult, SimpleFormat,
};
use amethyst_core::specs::prelude::{Component, DenseVecStorage, VecStorage};

use crate::Texture;

/// An asset handle to sprite sheet metadata.
pub type SpriteSheetHandle = Handle<SpriteSheet>;

/// Meta data for a sprite sheet texture.
///
/// Contains a handle to the texture and the sprite coordinates on the texture.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteSheet {
    /// `Texture` handle of the spritesheet texture
    pub texture: Handle<Texture>,
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

/// Information about whether or not a texture should be flipped
/// when rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Flipped {
    /// Don't flip the texture
    None,
    /// Flip the texture horizontally
    Horizontal,
    /// Flip the texture vertically
    Vertical,
    /// Flip the texture in both orientations
    Both,
}

impl Component for Flipped {
    type Storage = DenseVecStorage<Self>;
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

impl Sprite {
    /// Creates a `Sprite` from pixel values.
    ///
    /// This function expects pixel coordinates -- starting from the top left of the image. X
    /// increases to the right, Y increases downwards. Texture coordinates are calculated from the
    /// pixel values.
    ///
    /// # Parameters
    ///
    /// * `image_w`: Width of the full sprite sheet.
    /// * `image_h`: Height of the full sprite sheet.
    /// * `sprite_w`: Width of the sprite.
    /// * `sprite_h`: Height of the sprite.
    /// * `pixel_left`: Pixel X coordinate of the left side of the sprite.
    /// * `pixel_top`: Pixel Y coordinate of the top of the sprite.
    /// * `offsets`: Number of pixels to shift the sprite to the left and down relative to the
    ///              entity.
    pub fn from_pixel_values(
        image_w: u32,
        image_h: u32,
        sprite_w: u32,
        sprite_h: u32,
        pixel_left: u32,
        pixel_top: u32,
        offsets: [f32; 2],
    ) -> Sprite {
        let image_w = image_w as f32;
        let image_h = image_h as f32;
        let offsets = [offsets[0] as f32, offsets[1] as f32];

        let pixel_right = (pixel_left + sprite_w) as f32;
        let pixel_bottom = (pixel_top + sprite_h) as f32;
        let pixel_left = pixel_left as f32;
        let pixel_top = pixel_top as f32;

        // Texture coordinates are expressed as fractions of the position on the image.
        // Y axis texture coordinates start at the bottom of the image, so we have to invert them.
        //
        // For pixel perfect result, the sprite border must be rendered exactly at
        // screen pixel border or use nearest-neighbor sampling.
        // <http://www.mindcontrol.org/~hplus/graphics/opengl-pixel-perfect.html>
        // NOTE: Maybe we should provide an option to round coordinates from `Transform`
        // to nearest integer in `DrawFlat2D` pass before rendering.
        let left = (pixel_left) / image_w;
        let right = (pixel_right) / image_w;
        let top = (image_h - pixel_top) / image_h;
        let bottom = (image_h - pixel_bottom) / image_h;

        let tex_coords = TextureCoordinates {
            left,
            right,
            top,
            bottom,
        };

        Sprite {
            width: sprite_w as f32,
            height: sprite_h as f32,
            offsets,
            tex_coords,
        }
    }
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
/// render textures to quads. This struct carries the information necessary for the draw2dflat pass.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteRender {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: SpriteSheetHandle,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}

impl Component for SpriteRender {
    type Storage = VecStorage<Self>;
}

/// Structure acting as scaffolding for serde when loading a spritesheet file.
/// Positions originate in the top-left corner (bitmap image convention).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SpritePosition {
    /// Horizontal position of the sprite in the sprite sheet
    pub x: u32,
    /// Vertical position of the sprite in the sprite sheet
    pub y: u32,
    /// Width of the sprite
    pub width: u32,
    /// Height of the sprite
    pub height: u32,
    /// Number of pixels to shift the sprite to the left and down relative to the entity holding it
    pub offsets: Option<[f32; 2]>,
}

/// Structure acting as scaffolding for serde when loading a spritesheet file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SerializedSpriteSheet {
    /// Width of the sprite sheet
    pub spritesheet_width: u32,
    /// Height of the sprite sheet
    pub spritesheet_height: u32,
    /// Description of the sprites
    pub sprites: Vec<SpritePosition>,
}

/// Allows loading of sprite sheets in RON format.
///
/// This format allows to conveniently load a sprite sheet from a RON file.
///
/// Example:
/// ```text,ignore
/// (
///     // Width of the sprite sheet
///     spritesheet_width: 48.0,
///     // Height of the sprite sheet
///     spritesheet_height: 16.0,
///     // List of sprites the sheet holds
///     sprites: [
///         (
///             // Horizontal position of the sprite in the sprite sheet
///             x: 0.0,
///             // Vertical position of the sprite in the sprite sheet
///             y: 0.0,
///             // Width of the sprite
///             width: 16.0,
///             // Height of the sprite
///             height: 16.0,
///             // Number of pixels to shift the sprite to the left and down relative to the entity holding it when rendering
///             offsets: (0.0, 0.0), // This is optional and defaults to (0.0, 0.0)
///         ),
///         (
///             x: 16.0,
///             y: 0.0,
///             width: 32.0,
///             height: 16.0,
///         ),
///     ],
/// )
/// ```
///
/// Such a spritesheet description can be loaded using a `Loader` by passing it the handle of the corresponding loaded texture.
/// ```rust,no_run
/// # use amethyst_assets::{Loader, AssetStorage};
/// # use amethyst_renderer::{SpriteSheetFormat, SpriteSheet, Texture, PngFormat, TextureMetadata};
/// #
/// # fn load_sprite_sheet() {
/// #   let world = amethyst_core::specs::World::new(); // Normally, you would use Amethyst's world
/// #   let loader = world.read_resource::<Loader>();
/// #   let spritesheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
/// #   let texture_storage = world.read_resource::<AssetStorage<Texture>>();
/// let texture_handle = loader.load(
///     "my_texture.png",
///     PngFormat,
///     TextureMetadata::srgb(),
///     (),
///     &texture_storage,
/// );
/// let spritesheet_handle = loader.load(
///     "my_spritesheet.ron",
///     SpriteSheetFormat,
///     texture_handle,
///     (),
///     &spritesheet_storage,
/// );
/// # }
/// ```
#[derive(Clone, Deserialize, Serialize)]
pub struct SpriteSheetFormat;

impl SimpleFormat<SpriteSheet> for SpriteSheetFormat {
    const NAME: &'static str = "SPRITE_SHEET";

    type Options = Handle<Texture>;

    fn import(&self, bytes: Vec<u8>, texture: Self::Options) -> AssetsResult<SpriteSheet> {
        let sheet: SerializedSpriteSheet = from_ron_bytes(&bytes).map_err(|_| {
            AssetsError::from_kind(AssetsErrorKind::Format(
                "Failed to parse Ron file for SpriteSheet",
            ))
        })?;
        let mut sprites: Vec<Sprite> = Vec::with_capacity(sheet.sprites.len());
        for sp in sheet.sprites {
            let sprite = Sprite::from_pixel_values(
                sheet.spritesheet_width as u32,
                sheet.spritesheet_height as u32,
                sp.width as u32,
                sp.height as u32,
                sp.x as u32,
                sp.y as u32,
                sp.offsets.unwrap_or([0.0; 2]),
            );
            sprites.push(sprite);
        }
        Ok(SpriteSheet { texture, sprites })
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

    #[test]
    fn sprite_from_pixel_values_calculates_pixel_perfect_coordinates() {
        let image_w = 30;
        let image_h = 40;
        let sprite_w = 10;
        let sprite_h = 20;
        let pixel_left = 0;
        let pixel_top = 20;
        let offsets = [-5.0, -10.0];

        assert_eq!(
            Sprite::from((
                (10., 20.),                     // Sprite w and h
                [-5., -10.],                    // Offsets
                [0., 10. / 30., 0., 20. / 40.], // Texture coordinates
            )),
            Sprite::from_pixel_values(
                image_w, image_h, sprite_w, sprite_h, pixel_left, pixel_top, offsets
            )
        );
    }
}
