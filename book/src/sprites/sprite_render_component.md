# `SpriteRender` Component

After loading the `SpriteSheet`, you need to attach it to an entity using the `SpriteRender` component and indicate which sprite to draw. The `SpriteRender` component looks like this:

```rust,ignore
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteRender {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: SpriteSheetHandle,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
    /// Whether the sprite should be flipped horizontally
    pub flip_horizontal: bool,
    /// Whether the sprite should be flipped vertically
    pub flip_vertical: bool,
}
```

The `flip_*` fields indicate whether the sprite to be drawn should be flipped. The sprite number is the index of the sprite loaded in the sprite sheet. What's left is the `SpriteSheetHandle`.

In the previous section you wrote a function that returns a `SpriteSheet`. This can be turned into a `SpriteSheetHandle` using the `Loader` resource as follows:

```rust,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Loader};
# use amethyst::prelude::*;
use amethyst::renderer::{
    MaterialTextureSet, SpriteSheet, SpriteSheetHandle, TextureHandle,
};

# pub fn load_texture<N>(name: N, world: &World) -> TextureHandle
# where
#     N: Into<String>,
# {
#     unimplemented!();
# }
#
# pub fn load_sprite_sheet(texture_id: u64) -> SpriteSheet {
#     unimplemented!();
# }
#[derive(Debug)]
struct ExampleState;

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, mut data: StateData<GameData>) {
#         let texture_handle = load_texture("texture/sprite_sheet.png", &data.world);
#         let texture_id = 0;
#         data.world
#             .write_resource::<MaterialTextureSet>()
#             .insert(texture_id, texture_handle);
        // ...

        let sprite_sheet = load_sprite_sheet(texture_id);
        let sprite_sheet_handle = {
            let loader = data.world.read_resource::<Loader>();
            loader.load_from_data(
                sprite_sheet,
                (),
                &data.world.read_resource::<AssetStorage<SpriteSheet>>(),
            )
        };
    }
}
#
# fn main() {}
```

Cool, finally we have all the parts, let's build a `SpriteRender` and attach it to an entity:

```rust,no_run,noplaypen
# extern crate amethyst;
# use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::cgmath::Vector3;
use amethyst::core::transform::Transform;
# use amethyst::prelude::*;
use amethyst::renderer::{
    MaterialTextureSet, ScreenDimensions, SpriteRender, SpriteSheet,
    SpriteSheetHandle, TextureHandle, Transparent
};

# pub fn load_texture<N>(name: N, world: &World) -> TextureHandle
# where
#     N: Into<String>,
# {
#     unimplemented!();
# }
#
# pub fn load_sprite_sheet(texture_id: u64) -> SpriteSheet {
#     unimplemented!();
# }
#[derive(Debug)]
struct ExampleState;

impl<'a, 'b> SimpleState<'a, 'b> for ExampleState {
    fn on_start(&mut self, mut data: StateData<GameData>) {
#         let texture_handle = load_texture("texture/sprite_sheet.png", &data.world);
#         let texture_id = 0;
#         data.world
#             .write_resource::<MaterialTextureSet>()
#             .insert(texture_id, texture_handle);
# 
#         let sprite_sheet = load_sprite_sheet(texture_id);
#         let sprite_sheet_handle = {
#             let loader = data.world.read_resource::<Loader>();
#             loader.load_from_data(
#                 sprite_sheet,
#                 (),
#                 &data.world.read_resource::<AssetStorage<SpriteSheet>>(),
#             )
#         };
        // ...

        self.initialize_sprite(&mut data.world, sprite_sheet_handle);
    }
}

impl ExampleState {
    fn initialize_sprite(
        &mut self,
        world: &mut World,
        sprite_sheet_handle: SpriteSheetHandle,
    ) {
        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        // Move the sprite to the middle of the window
        let mut sprite_transform = Transform::default();
        sprite_transform.translation = Vector3::new(width / 2., height / 2., 0.);

        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet_handle,
            sprite_number: 0, // First sprite
            flip_horizontal: false,
            flip_vertical: false,
        };

        world
            .create_entity()
            .with(sprite_render)
            .with(sprite_transform)
            .with(Transparent) // If your sprite is transparent
            .build();
    }
}
#
# fn main() {}
```

Got that? Sweet!
