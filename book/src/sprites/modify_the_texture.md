# Modify The `Texture`

The colors of the sprite will show up exactly as in the source file,
but sometimes being able to slightly modify the overall color
is useful - for instance, coloring an angry enemy more red, or
making a frozen enemy blue. Amethyst has a [`Component`][doc_component] called
[`Tint`][doc_tint] to do this.

To use [`Tint`][doc_tint], register [`Tint`][doc_tint] as a new
[`Component`][doc_component] with the world and build it as part of the entity.
[`Tint`][doc_tint] will multiply the color values of the sprite by its
own values, so a [`Tint`][doc_tint] with a white color will have no
effect on the sprite.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::assets::{AssetStorage, Loader, Handle};
use amethyst::core::transform::Transform;
# use amethyst::prelude::*;
use amethyst::renderer::{
    palette::Srgba,
    resources::Tint,
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
        // ..

#         let (width, height) = {
#             let dim = world.read_resource::<ScreenDimensions>();
#             (dim.width(), dim.height())
#         };
#
#         // Move the sprite to the middle of the window
#         let mut sprite_transform = Transform::default();
#         sprite_transform.set_translation_xyz(width / 2., height / 2., 0.);
#
#         let sprite_render = SpriteRender::new(sprite_sheet_handle, 0);  // First sprite

        // White shows the sprite as normal.
        // You can change the color at any point to modify the sprite's tint.
        let tint = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

        world
            .create_entity()
            .with(sprite_render)
            .with(sprite_transform)
            .with(tint)
            .build();
    }
}
#
# fn main() {}
```

[doc_tint]: https://docs.amethyst.rs/stable/amethyst_rendy/resources/struct.Tint.html
[doc_component]: https://docs.amethyst.rs/stable/specs/trait.Component.html
