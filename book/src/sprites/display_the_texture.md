# Display The `Texture`

With a [`DrawFlat2D`][doc_drawflat2d] render pass set up and a loaded texture, it's already possible to render the full texture. The [`TextureHandle`][doc_tex_handle] itself implements [`Component`][doc_component], so you can just attach the [`Handle`][doc_handle] to an entity and it'll show up!

For anything rendered by the [`DrawFlat2D`][doc_drawflat2d] pass, it's also possible to optionally attach a [`Flipped`][doc_flipped] component to the entity, which will signal to the renderer that you want to flip your texture horizontally or vertically when rendering.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::Transform;
use amethyst::prelude::*;
use amethyst::renderer::{Flipped, PngFormat, Texture, TextureMetadata, TextureHandle};

# pub fn load_texture<N>(name: N, world: &mut World) -> TextureHandle
# where
#    N: Into<String>,
# {
#     let loader = world.read_resource::<Loader>();
#     loader.load(
#         name,
#         PngFormat,
#         TextureMetadata::srgb(),
#         (),
#         &world.read_resource::<AssetStorage<Texture>>(),
#     )
# }

// ...

fn init_image(world: &mut World, texture_handle: &TextureHandle) {
    // Add a transform component to give the image a position
    let mut transform = Transform::default();
    transform.set_x(0.0);
    transform.set_y(0.0);

    world
        .create_entity()
        .with(transform)
        .with(texture_handle.clone()) // Use the texture handle as a component
        .with(Flipped::Horizontal) // Flip the texture horizontally
        .build();
}

#[derive(Debug)]
struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let texture_handle = load_texture("texture/sprite_sheet.png", world);

        // show the image!
        init_image(world, &texture_handle);
    }
}
```

[doc_drawflat2d]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.DrawFlat2D.html
[doc_tex_handle]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/type.TextureHandle.html
[doc_component]: https://www.amethyst.rs/doc/latest/doc/specs/trait.Component.html
[doc_handle]: https://www.amethyst.rs/doc/latest/doc/amethyst_assets/struct.Handle.html
[doc_flipped]: https://www.amethyst.rs/doc/latest/doc/amethyst_renderer/struct.Flipped.html
