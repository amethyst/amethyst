# `SpriteRender` Component

After loading the `SpriteSheet`, you need to attach it to an entity using the `SpriteRender` component and indicate which sprite to draw. The `SpriteRender` component looks like this:

```rust,ignore
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteRender {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: Handle<SpriteSheet>,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}
```

The sprite number is the index of the sprite loaded in the sprite sheet. What's left is the `Handle<SpriteSheet>`.

In the previous section you wrote a function that returns a `SpriteSheet`. This can be turned into a `Handle<SpriteSheet>` using the `Loader` resource as follows:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Loader, Handle};
# use amethyst::prelude::*;
use amethyst::renderer::{SpriteSheet, Texture};

# pub fn load_texture<N>(name: N, world: &World) -> Handle<Texture>
# where
#     N: Into<String>,
# {
#     unimplemented!();
# }
#
# pub fn load_sprite_sheet(texture: Handle<Texture>) -> SpriteSheet {
#     unimplemented!();
# }
#[derive(Debug)]
struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
#         let texture_handle = load_texture("texture/sprite_sheet.png", &data.world);
        // ...

        let sprite_sheet = load_sprite_sheet(texture_handle);
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

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::assets::{AssetStorage, Loader, Handle};
use amethyst::core::transform::Transform;
# use amethyst::prelude::*;
use amethyst::renderer::{
    SpriteRender, SpriteSheet,
    Texture, Transparent
};
use amethyst::window::ScreenDimensions;

# pub fn load_texture<N>(name: N, world: &World) -> Handle<Texture>
# where
#     N: Into<String>,
# {
#     unimplemented!();
# }
#
# pub fn load_sprite_sheet(texture: Handle<Texture>) -> SpriteSheet {
#     unimplemented!();
# }
#[derive(Debug)]
struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
#         let texture_handle = load_texture("texture/sprite_sheet.png", &data.world);
#
#         let sprite_sheet = load_sprite_sheet(texture_handle);
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
        sprite_sheet_handle: Handle<SpriteSheet>,
    ) {
        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        // Move the sprite to the middle of the window
        let mut sprite_transform = Transform::default();
        sprite_transform.set_translation_xyz(width / 2., height / 2., 0.);

        // 0 indicates the first sprite in the sheet.
        let sprite_render = SpriteRender::new(sprite_sheet_handle, 0);  // First sprite

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
