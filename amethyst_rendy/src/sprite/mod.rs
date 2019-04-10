use rendy::hal::Backend;
use ron::de::from_bytes as from_ron_bytes;
use serde::{Deserialize, Serialize};

use amethyst_assets::{Asset, Handle, SimpleFormat};
use amethyst_core::ecs::prelude::{Component, DenseVecStorage, VecStorage};
use amethyst_error::Error;
use crate::{error, types::Texture};

mod prefab;

/// An asset handle to sprite sheet metadata.
pub type SpriteSheetHandle<B: Backend> = Handle<SpriteSheet<B>>;

/// Active camera for the `DrawFlat2D` pass.
pub type SpriteCamera = crate::camera::ActiveCamera;

/// Meta data for a sprite sheet texture.
///
/// Contains a handle to the texture and the sprite coordinates on the texture.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteSheet<B: Backend> {
    /// `Texture` handle of the spritesheet texture
    pub texture: Handle<Texture<B>>,
    /// A list of sprites in this sprite sheet.
    pub sprites: Vec<Sprite>,
}

impl<B: Backend> Asset for SpriteSheet<B> {
    const NAME: &'static str = "renderer::SpriteSheet";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

//impl<B: Backend> From<SpriteSheet<B>> for Result<ProcessingState<SpriteSheet<B>>, Error> {
//    fn from(sprite_sheet: SpriteSheet<B>) -> Result<ProcessingState<SpriteSheet<B>>, Error> {
//        Ok(ProcessingState::Loaded(sprite_sheet))
//    }
//}

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
pub struct SpriteRender<B: Backend> {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: SpriteSheetHandle<B>,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}

impl<B: Backend> Component for SpriteRender<B> {
    type Storage = VecStorage<Self>;
}

/// Represents one sprite in `SpriteList`.
/// Positions originate in the top-left corner (bitmap image convention).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SpritePosition {
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

/// `SpriteList` controls how a sprite list is generated when using `Sprites::List` in a
/// `SpriteSheetPrefab`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SpriteList {
    /// Width of the texture in pixels.
    pub texture_width: u32,
    /// Height of the texture in pixels.
    pub texture_height: u32,
    /// Description of the sprites
    pub sprites: Vec<SpritePosition>,
}

/// `SpriteGrid` controls how a sprite grid is generated when using `Sprites::Grid` in a
/// `SpriteSheetPrefab`.
///
/// The number of columns in the grid must always be provided, and one of the other fields must also
/// be provided. The grid will be layout row major, starting with the sprite in the upper left corner,
/// and ending with the sprite in the lower right corner. For example a grid with 2 rows and 4 columns
/// will have the order below for the sprites.
///
/// ```text
/// |---|---|---|---|
/// | 0 | 1 | 2 | 3 |
/// |---|---|---|---|
/// | 4 | 5 | 6 | 7 |
/// |---|---|---|---|
/// ```
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SpriteGrid {
    /// Width of the texture in pixels.
    pub texture_width: u32,
    /// Height of the texture in pixels.
    pub texture_height: u32,
    /// Specifies the number of columns in the spritesheet, this value must always be given.
    pub columns: u32,
    /// Specifies the number of rows in the spritesheet. If this is not given it will be calculated
    /// using either `sprite_count` (`sprite_count / columns`), or `cell_size` (`sheet_size / cell_size`).
    pub rows: Option<u32>,
    /// Specifies the number of sprites in the spritesheet. If this is not given it will be
    /// calculated using `rows` (`columns * rows`).
    pub sprite_count: Option<u32>,
    /// Specifies the size of the individual sprites in the spritesheet in pixels. If this is not
    /// given it will be calculated using the spritesheet size, `columns` and `rows`.
    /// Tuple order is `(width, height)`.
    pub cell_size: Option<(u32, u32)>,
    /// Specifies the position of the grid on a texture. If this is not given it will be set to (0, 0).
    /// Positions originate in the top-left corner (bitmap image convention).
    pub position: Option<(u32, u32)>,
}

/// Defined the sprites that are part of a `SpriteSheetPrefab`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Sprites {
    /// A list of sprites
    List(SpriteList),
    /// Generate a grid sprite list, see `SpriteGrid` for more information.
    Grid(SpriteGrid),
}

impl Sprites {
    fn build_sprites(&self) -> Vec<Sprite> {
        match self {
            Sprites::List(list) => list.build_sprites(),
            Sprites::Grid(grid) => grid.build_sprites(),
        }
    }
}

impl SpriteList {
    /// Creates a `Vec<Sprite>` from `SpriteList`.
    pub fn build_sprites(&self) -> Vec<Sprite> {
        self.sprites
            .iter()
            .map(|pos| {
                Sprite::from_pixel_values(
                    self.texture_width,
                    self.texture_height,
                    pos.width,
                    pos.height,
                    pos.x,
                    pos.y,
                    pos.offsets.unwrap_or([0.0; 2]),
                )
            })
            .collect()
    }
}

impl SpriteGrid {
    /// The width of the part of the texture that the sprites reside on
    fn sheet_width(&self) -> u32 {
        self.texture_width - self.position().0
    }

    /// The height of the part of the texture that the sprites reside on
    fn sheet_height(&self) -> u32 {
        self.texture_height - self.position().1
    }

    fn rows(&self) -> u32 {
        self.rows.unwrap_or_else(|| {
            self.sprite_count
                .map(|c| {
                    if (c % self.columns) == 0 {
                        (c / self.columns)
                    } else {
                        (c / self.columns) + 1
                    }
                })
                .or_else(|| self.cell_size.map(|(_, y)| (self.sheet_height() / y)))
                .unwrap_or(1)
        })
    }

    fn sprite_count(&self) -> u32 {
        self.sprite_count
            .unwrap_or_else(|| self.columns * self.rows())
    }

    fn cell_size(&self) -> (u32, u32) {
        self.cell_size.unwrap_or_else(|| {
            (
                (self.sheet_width() / self.columns),
                (self.sheet_height() / self.rows()),
            )
        })
    }

    fn position(&self) -> (u32, u32) {
        self.position.unwrap_or((0, 0))
    }

    /// Creates a `Vec<Sprite>` from `SpriteGrid`.
    pub fn build_sprites(&self) -> Vec<Sprite> {
        let rows = self.rows();
        let sprite_count = self.sprite_count();
        let cell_size = self.cell_size();
        let position = self.position();
        if (self.columns * cell_size.0) > self.sheet_width() {
            log::warn!(
                "Grid spritesheet contains more columns than can fit in the given width: {} * {} > {} - {}",
                self.columns,
                cell_size.0,
                self.texture_width,
                position.0
            );
        }
        if (rows * cell_size.1) > self.sheet_height() {
            log::warn!(
                "Grid spritesheet contains more rows than can fit in the given height: {} * {} > {} - {}",
                rows,
                cell_size.1,
                self.texture_height,
                position.1
            );
        }
        (0..sprite_count)
            .map(|cell| {
                let row = cell / self.columns;
                let column = cell - (row * self.columns);
                let x = column * cell_size.0 + position.0;
                let y = row * cell_size.1 + position.1;
                Sprite::from_pixel_values(
                    self.texture_width,
                    self.texture_height,
                    cell_size.0,
                    cell_size.1,
                    x,
                    y,
                    [0.0; 2],
                )
            })
            .collect()
    }
}

/// Allows loading of sprite sheets in RON format.
///
/// This format allows to conveniently load a sprite sheet from a RON file.
///
/// Example:
/// ```text,ignore
/// (
///     // Width of the texture
///     texture_width: 48,
///     // Height of the texture
///     texture_height: 16,
///     // List of sprites the sheet holds
///     sprites: [
///         (
///             // Horizontal position of the sprite in the sprite sheet
///             x: 0,
///             // Vertical position of the sprite in the sprite sheet
///             y: 0,
///             // Width of the sprite
///             width: 16,
///             // Height of the sprite
///             height: 16,
///             // Number of pixels to shift the sprite to the left and down relative to the entity holding it when rendering
///             offsets: (0.0, 0.0), // This is optional and defaults to (0.0, 0.0)
///         ),
///         (
///             x: 16,
///             y: 0,
///             width: 32,
///             height: 16,
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
/// #   let world = amethyst_core::ecs::World::new(); // Normally, you would use Amethyst's world
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

impl<B: Backend> SimpleFormat<SpriteSheet<B>> for SpriteSheetFormat {
    const NAME: &'static str = "SPRITE_SHEET";

    type Options = Handle<Texture<B>>;

    fn import(&self, bytes: Vec<u8>, texture: Self::Options) -> Result<SpriteSheet<B>, Error> {
        let sprite_list: SpriteList =
            from_ron_bytes(&bytes).map_err(|_| error::Error::LoadSpritesheetError)?;

        Ok(SpriteSheet {
            texture,
            sprites: sprite_list.build_sprites(),
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
