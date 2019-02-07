use amethyst::{
    assets::Handle,
    renderer::{Sprite, SpriteSheet, Texture},
};

use log::debug;

use crate::sprite;

// Please note that this is the advanced way to load a SpriteSheet in Amethyst.
// Please consult the `pong` example if you would prefer an easier solution.
// Alternatively, the Amethyst book documents all the loading methods available.

/// Loads a sprite sheet from the assets folder.
///
/// # Parameters:
///
/// * `texture`: Sprite sheet's texture handle
/// * `definition`: Definition of the sprite layout on the sprite sheet.
pub fn load(texture: Handle<Texture>, definition: &sprite::SpriteSheetDefinition) -> SpriteSheet {
    let mut sprites = Vec::with_capacity((definition.row_count * definition.column_count) as usize);
    let (offset_w, offset_h) = offset_distances(&definition);
    let (image_w, image_h) = (
        offset_w * definition.column_count,
        offset_h * definition.row_count,
    );

    for row in 0..definition.row_count {
        for col in 0..definition.column_count {
            // Sprites are numbered in the following pattern:
            //
            //  0  1  2  3  4
            //  5  6  7  8  9
            // 10 11 12 13 14
            // 15 16 17 18 19

            let pixel_left = offset_w * col;
            let pixel_top = offset_h * row;
            let sprite = Sprite::from_pixel_values(
                image_w,
                image_h,
                definition.sprite_w,
                definition.sprite_h,
                pixel_left,
                pixel_top,
                [0.0; 2],
            );

            let sprite_number = row * definition.column_count + col;
            debug!("{}: Sprite: {:?}", sprite_number, &sprite);

            sprites.push(sprite);
        }
    }

    SpriteSheet { texture, sprites }
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
fn offset_distances(definition: &sprite::SpriteSheetDefinition) -> (u32, u32) {
    if definition.has_border {
        (definition.sprite_w + 1, definition.sprite_h + 1)
    } else {
        (definition.sprite_w, definition.sprite_h)
    }
}
