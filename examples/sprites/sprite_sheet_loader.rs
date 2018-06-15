use amethyst::renderer::{Sprite, SpriteSheet};

use sprite;

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

    // Push the rows in reverse order because the texture coordinates are treated as beginning
    // from the bottom of the image, whereas for this example I want the top left sprite to be
    // the first sprite
    for row in (0..definition.row_count).rev() {
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
                offset_x,
                offset_y,
                offset_x + definition.sprite_w,
                offset_y + definition.sprite_h,
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
/// * `pixel_left`: Pixel X coordinate of the left side of the sprite.
/// * `pixel_top`: Pixel Y coordinate of the top of the sprite.
/// * `pixel_right`: Pixel X coordinate of the right side of the sprite.
/// * `pixel_bottom`: Pixel Y coordinate of the bottom of the sprite.
fn create_sprite(
    image_w: f32,
    image_h: f32,
    pixel_left: f32,
    pixel_top: f32,
    pixel_right: f32,
    pixel_bottom: f32,
) -> Sprite {
    // Texture coordinates are expressed as fractions of the position on the image.
    let left = pixel_left / image_w;
    let top = pixel_top / image_h;
    let right = pixel_right / image_w;
    let bottom = pixel_bottom / image_h;

    Sprite {
        left,
        top,
        right,
        bottom,
    }
}
