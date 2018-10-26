use amethyst::renderer::{Sprite, SpriteSheet, TextureCoordinates};

use sprite;

// Please note that this is the advanced way to load a SpriteSheet in Amethyst.
// Please consult the `pong` example if you would prefer an easier solution.
// Alternatively, the Amethyst book documents all the loading methods available.

/// Loads a sprite sheet from the assets folder.
///
/// # Parameters:
///
/// * `texture_id`: Id of the sprite sheet's texture in the `MaterialTextureSet`.
/// * `definition`: Definition of the sprite layout on the sprite sheet.
pub fn load(texture_id: u64, definition: &sprite::SpriteSheetDefinition) -> SpriteSheet {
    let mut sprites = Vec::with_capacity(definition.row_count * definition.column_count);
    let (offset_w, offset_h) = offset_distances(&definition);
    let (image_w, image_h) = (
        offset_w * definition.column_count as f32,
        offset_h * definition.row_count as f32,
    );

    for row in 0..definition.row_count {
        for col in 0..definition.column_count {
            // Sprites are numbered in the following pattern:
            //
            //  0  1  2  3  4
            //  5  6  7  8  9
            // 10 11 12 13 14
            // 15 16 17 18 19

            let offset_x = offset_w * col as f32;
            let offset_y = offset_h * row as f32;
            let sprite = create_sprite(
                image_w,
                image_h,
                definition.sprite_w,
                definition.sprite_h,
                offset_x,
                offset_y,
            );

            let sprite_number = row * definition.column_count + col;
            debug!("{}: Sprite: {:?}", sprite_number, &sprite);

            sprites.push(sprite);
        }
    }

    SpriteSheet {
        texture_id,
        sprites,
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
/// * `definition`: Sprite sheet definition.
fn offset_distances(definition: &sprite::SpriteSheetDefinition) -> (f32, f32) {
    if definition.has_border {
        (definition.sprite_w + 1., definition.sprite_h + 1.)
    } else {
        (definition.sprite_w, definition.sprite_h)
    }
}

/// Returns a set of vertices that make up a rectangular mesh of the given size.
///
/// This function expects pixel coordinates -- starting from the top left of the image. X increases
/// to the right, Y increases downwards.
///
/// # Parameters
///
/// * `image_w`: Width of the full sprite sheet.
/// * `image_h`: Height of the full sprite sheet.
/// * `sprite_w`: Width of the sprite.
/// * `sprite_h`: Height of the sprite.
/// * `pixel_left`: Pixel X coordinate of the left side of the sprite.
/// * `pixel_top`: Pixel Y coordinate of the top of the sprite.
fn create_sprite(
    image_w: f32,
    image_h: f32,
    sprite_w: f32,
    sprite_h: f32,
    pixel_left: f32,
    pixel_top: f32,
) -> Sprite {
    let pixel_right = pixel_left + sprite_w;
    let pixel_bottom = pixel_top + sprite_h;

    // Texture coordinates are expressed as fractions of the position on the image.
    let left = pixel_left / image_w;
    let right = pixel_right / image_w;
    let top = 1.0 - pixel_top / image_h;
    let bottom = 1.0 - pixel_bottom / image_h;

    let tex_coords = TextureCoordinates {
        left,
        right,
        bottom,
        top,
    };

    Sprite {
        width: sprite_w,
        height: sprite_h,
        offsets: [sprite_w / 2.0, sprite_h / 2.0],
        tex_coords,
    }
}
