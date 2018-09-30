# Define The `SpriteSheet`

With the texture loaded, Amethyst still needs to know *where the sprites are* on the image. Importantly, **we use pixel coordinates as well as texture coordinates** to define the sprite layout. Pixel coordinates indicate the dimensions of the sprite to draw on screen; texture coordinates indicate which part of the image contains the sprite, and are expressed as a proportion of the image.

The following table lists the differences between the coordinate systems:

| Pixel coordinates                     | Texture coordinates                       |
| ------------------------------------- | ----------------------------------------- |
| Begin at the top left of the image    | Begin at the bottom left of the image     |
| Increase to the right and down        | Increase to the right and up              |
| Range from 0 to (width or height - 1) | Range from 0.0 to 1.0                     |
| Use pixel values at exact coordinates | Takes average value of surrounding pixels |

In Amethyst, pixel dimensions and texture coordinates are stored in the `Sprite` struct. Since texture coordinates can be derived from pixel coordinates, Amethyst provides the `Sprite::from_pixel_values` function to create a `Sprite`.

The following snippet shows you how to naively define a `SpriteSheet`. In a real application, you would typically load this from configuration:

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::renderer::{Sprite, SpriteSheet, TextureCoordinates};

/// Returns a `SpriteSheet`.
///
/// # Parameters
///
/// * `texture_id`: ID of the texture in the `MaterialTextureSet`.
pub fn load_sprite_sheet(texture_id: u64) -> SpriteSheet {
    let sprite_count = 1; // number of sprites
    let mut sprites = Vec::with_capacity(sprite_count);

    let image_w = 100;
    let image_h = 20;
    let sprite_w = 10;
    let sprite_h = 10;

    // Here we are loading the 5th sprite on the bottom row.
    let offset_x = 50; // 5th sprite * 10 pixel sprite width
    let offset_y = 10; // Second row (1) * 10 pixel sprite height
    let offsets = [5; 2]; // Align the sprite with the middle of the entity.

    let sprite = Sprite::from_pixel_values(
        image_w, image_h, sprite_w, sprite_h, offset_x, offset_y, offsets,
    );
    sprites.push(sprite);

    SpriteSheet {
        texture_id,
        sprites,
    }
}
```
