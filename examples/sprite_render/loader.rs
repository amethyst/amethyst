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
        for row in 0..metadata.row_count {
            for col in 0..metadata.column_count {
                // Sprites are numbered in the following pattern:
                //
                //  0  1  2  3  4
                //  5  6  7  8  9
                // 10 11 12 13 14
                // 15 16 17 18 19
                // let _sprite_number = row * metadata.row_count + col;

                let offset_x = offset_w * col as f32;
                let offset_y = offset_h * row as f32;
                let vertices = create_sprite_vertices(
                    offset_x,
                    offset_y,
                    offset_x + metadata.sprite_w,
                    offset_y + metadata.sprite_h,
                );

                let mesh_handle = loader.load_from_data(
                    vertices.into(),
                    (),
                    &world.read_resource::<AssetStorage<Mesh>>(),
                );

                sprite_meshes.push(mesh_handle);
            }
        }

        // let vertices = create_sprite_vertices(0., 0., metadata.sprite_w, metadata.sprite_h);

        // let mesh_handle = loader.load_from_data(
        //     vertices.into(),
        //     (),
        //     &world.read_resource::<AssetStorage<Mesh>>(),
        // );

        // sprite_meshes.push(mesh_handle);

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
/// # Parameters
///
/// * `left`: x coordinate of the left side of the sprite.
/// * `bottom`: y coordinate of the bottom of the sprite.
/// * `right`: x coordinate of the right side of the sprite.
/// * `top`: y coordinate of the top of the sprite.
fn create_sprite_vertices(left: f32, bottom: f32, right: f32, top: f32) -> Vec<PosTex> {
    vec![
        PosTex {
            position: [left, bottom, 0.],
            tex_coord: [0., 0.],
        },
        PosTex {
            position: [right, bottom, 0.],
            tex_coord: [1., 0.],
        },
        PosTex {
            position: [left, top, 0.],
            tex_coord: [0., 1.],
        },
        PosTex {
            position: [right, top, 0.],
            tex_coord: [1., 1.],
        },
        PosTex {
            position: [left, top, 0.],
            tex_coord: [0., 1.],
        },
        PosTex {
            position: [right, bottom, 0.],
            tex_coord: [1., 0.],
        },
    ]
}
