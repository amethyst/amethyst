//! 2D Sprite Rendering implementation details.
use ron::de::from_bytes as from_ron_bytes;
use serde::{Deserialize, Serialize};

use crate::{error, types::Texture};
use amethyst_assets::{Asset, Format, Handle};
use amethyst_core::ecs::prelude::{Component, DenseVecStorage};
use amethyst_error::Error;

pub mod prefab;

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
    type HandleStorage = DenseVecStorage<Handle<Self>>;
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
        flip_horizontal: bool,
        flip_vertical: bool,
    ) -> Sprite {
        let image_w = image_w as f32;
        let image_h = image_h as f32;
        let offsets = [offsets[0] as f32, offsets[1] as f32];

        let pixel_right = (pixel_left + sprite_w) as f32;
        let pixel_bottom = (pixel_top + sprite_h) as f32;
        let pixel_left = pixel_left as f32;
        let pixel_top = pixel_top as f32;

        // Texture coordinates are expressed as fractions of the position on the image.
        //
        // For pixel perfect result, the sprite border must be rendered exactly at
        // screen pixel border or use nearest-neighbor sampling.
        // <http://www.mindcontrol.org/~hplus/graphics/opengl-pixel-perfect.html>
        // NOTE: Maybe we should provide an option to round coordinates from `Transform`
        // to nearest integer in `DrawFlat2D` pass before rendering.
        let left = (pixel_left) / image_w;
        let right = (pixel_right) / image_w;
        let top = (pixel_top) / image_h;
        let bottom = (pixel_bottom) / image_h;

        let (left, right) = if flip_horizontal {
            (right, left)
        } else {
            (left, right)
        };
        let (top, bottom) = if flip_vertical {
            (bottom, top)
        } else {
            (top, bottom)
        };

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

impl From<&TextureCoordinates> for [f32; 4] {
    fn from(item: &TextureCoordinates) -> Self {
        [item.left, item.right, item.bottom, item.top]
    }
}

/// Information for rendering a sprite.
///
/// Instead of using a `Mesh` on a `DrawFlat` render pass, we can use a simpler
/// set of shaders to render textures to quads. This struct carries the
/// information necessary for the draw2dflat pass.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteRender {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: Handle<SpriteSheet>,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}

impl SpriteRender {
    /// Create a new `SpriteRender`.
    pub fn new(sprite_sheet: Handle<SpriteSheet>, sprite_number: usize) -> SpriteRender {
        SpriteRender {
            sprite_sheet,
            sprite_number,
        }
    }
}

impl Component for SpriteRender {
    type Storage = DenseVecStorage<Self>;
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
    #[serde(default = "default_offsets")]
    pub offsets: Option<[f32; 2]>,
    /// Flip the sprite horizontally during rendering
    #[serde(default = "default_flip")]
    pub flip_horizontal: bool,
    /// Flip the sprite vertically during rendering
    #[serde(default = "default_flip")]
    pub flip_vertical: bool,
}

fn default_offsets() -> Option<[f32; 2]> {
    None
}

fn default_flip() -> bool {
    false
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
                    pos.flip_horizontal,
                    pos.flip_vertical,
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
                        c / self.columns
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
                    false,
                    false,
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
/// #![enable(implicit_some)]
/// List((
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
///             // Number of pixels to shift the sprite to the left and down relative to the
///             // entity holding it when rendering
///             offsets: (0.0, 0.0), // This is optional and defaults to (0.0, 0.0)
///         ),
///         (
///             x: 16,
///             y: 0,
///             width: 32,
///             height: 16,
///         ),
///     ],
/// ))
/// ```
///
/// Such a spritesheet description can be loaded using a `Loader` by passing it the handle of the corresponding loaded texture.
/// ```rust,no_run
/// # use amethyst_core::ecs::{World, WorldExt};
/// # use amethyst_assets::{Loader, AssetStorage};
/// # use amethyst_rendy::{sprite::{SpriteSheetFormat, SpriteSheet}, Texture, formats::texture::ImageFormat};
/// #
/// # fn load_sprite_sheet() {
/// #   let world = World::new(); // Normally, you would use Amethyst's world
/// #   let loader = world.read_resource::<Loader>();
/// #   let spritesheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
/// #   let texture_storage = world.read_resource::<AssetStorage<Texture>>();
/// let texture_handle = loader.load(
///     "my_texture.png",
///     ImageFormat(Default::default()),
///     (),
///     &texture_storage,
/// );
/// let spritesheet_handle = loader.load(
///     "my_spritesheet.ron",
///     SpriteSheetFormat(texture_handle),
///     (),
///     &spritesheet_storage,
/// );
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct SpriteSheetFormat(pub Handle<Texture>);

impl Format<SpriteSheet> for SpriteSheetFormat {
    fn name(&self) -> &'static str {
        "SPRITE_SHEET"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<SpriteSheet, Error> {
        let sprites: Sprites =
            from_ron_bytes(&bytes).map_err(error::Error::LoadSpritesheetError)?;

        Ok(SpriteSheet {
            texture: self.0.clone(),
            sprites: sprites.build_sprites(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Sprite, SpriteSheetFormat, TextureCoordinates};
    use crate::types::Texture;
    use amethyst_assets::Handle;

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
                [0., 10. / 30., 1., 20. / 40.], // Texture coordinates
            )),
            Sprite::from_pixel_values(
                image_w, image_h, sprite_w, sprite_h, pixel_left, pixel_top, offsets, false, false
            )
        );
    }
    fn create_texture() -> Handle<Texture> {
        use crate::formats::texture::TextureGenerator;
        use amethyst_assets::{AssetStorage, Loader};
        use rayon::ThreadPoolBuilder;
        use std::sync::Arc;

        let pool = Arc::new(ThreadPoolBuilder::new().build().expect("Invalid config"));
        let loader = Loader::new("/examples/assets", pool);
        let generator = TextureGenerator::Srgba(1.0, 1., 1., 1.);

        let storage: AssetStorage<Texture> = AssetStorage::default();

        loader.load_from_data(generator.data(), (), &storage)
    }
    #[test]
    fn sprite_sheet_loader_list() {
        use amethyst_assets::Format;

        let sprite_sheet_ron: String = "
#![enable(implicit_some)]
List((
    texture_width: 48,
    texture_height: 16,
    sprites: [
        (
            x: 0,
            y: 0,
            width: 16,
            height: 16,
        ),
        (
            x: 16,
            y: 0,
            width: 32,
            height: 16,
        ),
    ],
))"
        .to_string();

        let sprite_list_reference: Vec<Sprite> = vec![
            Sprite {
                width: 16.,
                height: 16.,
                offsets: [0., 0.],
                tex_coords: TextureCoordinates {
                    left: 0.,
                    right: 0.333_333_34,
                    bottom: 1.0,
                    top: 0.0,
                },
            },
            Sprite {
                width: 32.,
                height: 16.,
                offsets: [0., 0.],
                tex_coords: TextureCoordinates {
                    left: 0.333_333_34,
                    right: 1.0,
                    bottom: 1.0,
                    top: 0.0,
                },
            },
        ];

        let format = SpriteSheetFormat(create_texture());
        let sprite_list_loaded = format.import_simple(sprite_sheet_ron.into_bytes());
        let sprite_list = sprite_list_loaded.unwrap().sprites;
        assert_eq!(sprite_list_reference, sprite_list);
    }

    #[test]
    fn sprite_sheet_loader_grid() {
        use amethyst_assets::Format;

        let sprite_sheet_ron_rows: String = "
#![enable(implicit_some)]
Grid((
    texture_width: 48,
    texture_height: 16,
    columns: 2,
    rows: 1
))"
        .to_string();

        let sprite_sheet_ron_cells: String = "
#![enable(implicit_some)]
Grid((
    texture_width: 48,
    texture_height: 16,
    columns: 2,
    sprite_count: 2
))"
        .to_string();

        let sprite_sheet_ron_cell_size: String = "
#![enable(implicit_some)]
Grid((
    texture_width: 48,
    texture_height: 16,
    columns: 2,
    cell_size: (24, 16)
))"
        .to_string();

        let sprite_list_reference: Vec<Sprite> = vec![
            Sprite {
                width: 24.,
                height: 16.,
                offsets: [0., 0.],
                tex_coords: TextureCoordinates {
                    left: 0.,
                    right: 0.5,
                    bottom: 1.0,
                    top: 0.0,
                },
            },
            Sprite {
                width: 24.,
                height: 16.,
                offsets: [0., 0.],
                tex_coords: TextureCoordinates {
                    left: 0.5,
                    right: 1.0,
                    bottom: 1.0,
                    top: 0.0,
                },
            },
        ];
        let texture = create_texture();
        let format = SpriteSheetFormat(texture);
        {
            let sprite_list_loaded = format.import_simple(sprite_sheet_ron_rows.into_bytes());
            let sprite_list = sprite_list_loaded
                .expect("failed to parse sprite_sheet_ron_rows")
                .sprites;
            assert_eq!(
                sprite_list_reference, sprite_list,
                "we are testing row based grids"
            );
        }
        {
            let sprite_list_loaded = format.import_simple(sprite_sheet_ron_cells.into_bytes());
            let sprite_list = sprite_list_loaded
                .expect("failed to parse sprite_sheet_ron_cells")
                .sprites;
            assert_eq!(
                sprite_list_reference, sprite_list,
                "we are testing number of cell based grids"
            );
        }
        {
            let sprite_list_loaded = format.import_simple(sprite_sheet_ron_cell_size.into_bytes());
            let sprite_list = sprite_list_loaded
                .expect("failed to parse sprite_sheet_ron_cell_size")
                .sprites;
            assert_eq!(
                sprite_list_reference, sprite_list,
                "we are testing cell size based grids"
            );
        }
    }
}
