use amethyst::assets::{AssetStorage, Loader};
use amethyst::prelude::*;
use amethyst::renderer::{Material, MaterialDefaults, Mesh, PngFormat, PosTex, Texture};

use sprite;

#[derive(Debug)]
pub struct SpriteSheetLoader;

impl SpriteSheetLoader {
    /// Loads a PNG sprite sheet from the assets folder.
    ///
    /// It returns the mesh and texture so the caller can create an entity from it.
    pub fn load<N>(
        &self,
        name: N,
        metadata: sprite::Metadata,
        mut world: &mut World,
    ) -> sprite::Sheet
    where
        N: Into<String>,
    {
        let image = load_sheet_image(name, &mut world);

        let loader = world.read_resource::<Loader>();

        let mut sprite_meshes = Vec::with_capacity(metadata.row_count * metadata.column_count);
        let (offset_w, offset_h) = offset_distances(&metadata);
        let (image_w, image_h) = (
            offset_w * metadata.column_count as f32,
            offset_h * metadata.row_count as f32,
        );
        for row in 0..metadata.row_count {
            for col in 0..metadata.column_count {
                // Sprites are numbered in the following pattern:
                //
                //  0  1  2  3  4
                //  5  6  7  8  9
                // 10 11 12 13 14
                // 15 16 17 18 19

                let offset_x = offset_w * col as f32;
                let offset_y = offset_h * row as f32;
                let vertices = create_sprite_vertices(
                    image_w,
                    image_h,
                    metadata.sprite_w,
                    metadata.sprite_h,
                    offset_x,
                    offset_y,
                );

                let sprite_number = row * metadata.row_count + col;
                debug!("{}: Vertices: {:?}", sprite_number, &vertices);

                let mesh_handle = loader.load_from_data(
                    vertices.into(),
                    (),
                    &world.read_resource::<AssetStorage<Mesh>>(),
                );

                sprite_meshes.push(mesh_handle);
            }
        }

        sprite::Sheet::new(sprite_meshes, image)
    }
}

/// Returns a `Material` with the `albedo` referencing the loaded image data.
///
/// The `albedo` is the "natural colour" of the material under diffuse lighting.
///
/// # Parameters
///
/// * `name`: Path to the sprite sheet.
/// * `world`: `World` that stores resources.
fn load_sheet_image<N>(name: N, world: &mut World) -> Material
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    let sprite_sheet = loader.load(
        name,
        PngFormat,
        Default::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    );

    let mat_defaults = world.read_resource::<MaterialDefaults>();

    Material {
        albedo: sprite_sheet,
        ..mat_defaults.0.clone()
    }
}

/// Returns the pixel offset distances per sprite.
///
/// This is simply the sprite width and height if there is no border between sprites, or 1 added to
/// the width and height if there is a border. There is no leading border on the left or top of the
/// leftmost and topmost sprites.
///
/// # Parameters
///
/// * `metadata`: Sprite sheet metadata.
fn offset_distances(metadata: &sprite::Metadata) -> (f32, f32) {
    if metadata.has_border {
        (metadata.sprite_w + 1., metadata.sprite_h + 1.)
    } else {
        (metadata.sprite_w, metadata.sprite_h)
    }
}

/// Returns a set of vertices that make up a rectangular mesh of the given size.
///
/// Coordinates in this function are calculated from the top left of the image. X increases to the
/// right, Y increases downwards.
///
/// # Parameters
///
/// * `image_w`: Width of the full sprite sheet.
/// * `image_h`: Height of the full sprite sheet.
/// * `sprite_w`: Width of each sprite, excluding the border pixel if any.
/// * `sprite_h`: Height of each sprite, excluding the border pixel if any.
/// * `left`: X coordinate of the left side of the sprite.
/// * `top`: Y coordinate of the top of the sprite.
fn create_sprite_vertices(
    image_w: f32,
    image_h: f32,
    sprite_w: f32,
    sprite_h: f32,
    left: f32,
    top: f32,
) -> Vec<PosTex> {
    let right = left + sprite_w;
    let bottom = top + sprite_h;

    // Texture coordinates are expressed as fractions of the position on the image.
    let tex_coord_left = left / image_w;
    let tex_coord_right = right / image_w;
    let tex_coord_top = top / image_h;
    let tex_coord_bottom = bottom / image_h;

    vec![
        PosTex {
            position: [0., 0., 0.],
            tex_coord: [tex_coord_left, tex_coord_top],
        },
        PosTex {
            position: [sprite_w, 0., 0.],
            tex_coord: [tex_coord_right, tex_coord_top],
        },
        PosTex {
            position: [0., sprite_h, 0.],
            tex_coord: [tex_coord_left, tex_coord_bottom],
        },
        PosTex {
            position: [sprite_w, sprite_h, 0.],
            tex_coord: [tex_coord_right, tex_coord_bottom],
        },
        PosTex {
            position: [0., sprite_h, 0.],
            tex_coord: [tex_coord_left, tex_coord_bottom],
        },
        PosTex {
            position: [sprite_w, 0., 0.],
            tex_coord: [tex_coord_right, tex_coord_top],
        },
    ]
}
