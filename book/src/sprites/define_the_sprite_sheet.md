# Define The `SpriteSheet`

With the texture loaded, Amethyst still needs to know *where the sprites are* on the image.
There are two ways to load a sprite sheet definition: from a file or from code.

## Load the sheet from a file

The easiest way to load your sprites is to use a sprite sheet definition ron file.
Here is an example of such a definition file using a list of sprites:

```text,ignore
List((
    // Width of the texture used by the sprite sheet
    texture_width: 48,
    // Height of the texture used by the sprite sheet
    texture_height: 16,
    // List of sprites the sheet holds
    sprites: [
        (
            // Horizontal position of the sprite in the sprite sheet
            x: 0,
            // Vertical position of the sprite in the sprite sheet
            y: 0,
            // Width of the sprite
            width: 16,
            // Height of the sprite
            height: 16,
            // Number of pixels to shift the sprite to the left and down relative to the entity holding it when rendering
            offsets: Some((0.0, 0.0)), // This is optional and defaults to (0.0, 0.0)
        ),
        (
            x: 16,
            y: 0,
            width: 32,
            height: 16,
        ),
        // etc...
    ],
))
```

Or you can use a grid based definition, for example:

```text,ignore
Grid((
    // Width of the texture used by the sprite sheet
    texture_width: 48,
    // Height of the texture used by the sprite sheet
    texture_height: 16,
    // Specifies the number of columns in the sprite sheet
    columns: 2,
    // Specifies the number of sprites in the spritesheet.
    sprite_count: Some(2)
))
```

`Option` types need to be wrapped in the `Some` variant. For convenience, this can be left out if the line `#![enable(implicit_some)]` is added at the top of the definition file.  For example, `sprite_count: Some(2),` could be replaced by `sprite_count: 2,`.

For more information about list and grid based sprite sheets, including the types of their fields, see [`SpriteGrid`][doc_grid] or [`SpriteList`][doc_list].

Once you have ron file ready, you can load it using the texture handle of the sheet's image you loaded earlier:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::assets::{Loader, AssetStorage, Handle};
# use amethyst::ecs::{World, WorldExt};
# use amethyst::renderer::{SpriteSheetFormat, SpriteSheet, Texture};
#
# fn load_texture() -> Handle<Texture> {
#    unimplemented!()
# }
#
# fn load_sprite_sheet() {
#   let world = World::new();
#   let loader = world.read_resource::<Loader>();
#   let texture_handle = load_texture();
#   let spritesheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
let spritesheet_handle = loader.load(
    "my_spritesheet.ron",
    SpriteSheetFormat(texture_handle),
    (),
    &spritesheet_storage,
);
# }
```

This will get you the `Handle<SpriteSheet>` you will then use to draw the sprites.

## Load the sheet from code

While it is not the recommended way, it is also possible to manually build your sheet with code.

Importantly, **we use pixel coordinates as well as texture coordinates** to define the sprite layout. Pixel coordinates indicate the dimensions of the sprite to draw on screen; texture coordinates indicate which part of the image contains the sprite, and are expressed as a proportion of the image.

The following table lists the differences between the coordinate systems:

| Pixel coordinates                     | Texture coordinates                       |
| ------------------------------------- | ----------------------------------------- |
| Begin at the top left of the image    | Begin at the bottom left of the image     |
| Increase to the right and down        | Increase to the right and up              |
| Range from 0 to (width or height - 1) | Range from 0.0 to 1.0                     |

In Amethyst, pixel dimensions and texture coordinates are stored in the `Sprite` struct. Since texture coordinates can be derived from pixel coordinates, Amethyst provides the `Sprite::from_pixel_values` function to create a `Sprite`.

The following snippet shows you how to naively define a `SpriteSheet`. In a real application, you would typically use the sprite sheet from file feature, which is much more convenient.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::Handle;
use amethyst::renderer::{sprite::TextureCoordinates, Sprite, SpriteSheet, Texture};

/// Returns a `SpriteSheet`.
///
/// # Parameters
///
/// * `texture`: Handle of the texture.
pub fn load_sprite_sheet(texture: Handle<Texture>) -> SpriteSheet {
    let sprite_count = 1; // number of sprites
    let mut sprites = Vec::with_capacity(sprite_count);

    let image_w = 100;
    let image_h = 20;
    let sprite_w = 10;
    let sprite_h = 10;

    // Here we are loading the 5th sprite on the bottom row.
    let offset_x = 50; // 5th sprite * 10 pixel sprite width
    let offset_y = 10; // Second row (1) * 10 pixel sprite height
    let offsets = [5.0; 2]; // Align the sprite with the middle of the entity.

    let sprite = Sprite::from_pixel_values(
        image_w, image_h, sprite_w, sprite_h, offset_x, offset_y, offsets, false, false,
    );
    sprites.push(sprite);

    SpriteSheet {
        texture,
        sprites,
    }
}
```



[doc_grid]: https://docs.amethyst.rs/stable/amethyst_rendy/sprite/struct.SpriteGrid.html
[doc_list]: https://docs.amethyst.rs/stable/amethyst_rendy/sprite/struct.SpriteList.html