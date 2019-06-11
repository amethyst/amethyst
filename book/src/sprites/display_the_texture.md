# Display The `Texture`

With a [`DrawFlat2D`][doc_drawflat2d] render pass set up and a loaded texture, it's already possible to render the full texture. The [`TextureHandle`][doc_tex_handle] itself implements [`Component`][doc_component], so you can just attach the [`Handle`][doc_handle] to an entity and it'll show up!

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::Transform;
use amethyst::prelude::*;
use amethyst::renderer::{ImageFormat, Texture};

# pub fn load_texture<N>(name: N, world: &mut World) -> Handle<Texture>
# where
#    N: Into<String>,
# {
#     let loader = world.read_resource::<Loader>();
#     loader.load(
#         name,
#         ImageFormat::default(),
#         (),
#         &world.read_resource::<AssetStorage<Texture>>(),
#     )
# }

// ...

fn init_image(world: &mut World, texture_handle: &Handle<Texture>) {
    use amethyst::core::math::RealField;

    // Add a transform component to give the image a position
    let mut transform = Transform::default();
    transform.set_translation_x(0.0);
    transform.set_translation_y(0.0);
    
    // Flip horizontally
    transform.set_rotation_y_axis(f32::pi());

    world
        .create_entity()
        .with(transform)
        .with(texture_handle.clone()) // Use the texture handle as a component
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

[doc_drawflat2d]: https://docs-src.amethyst.rs/stable/amethyst_renderer/struct.DrawFlat2D.html
[doc_tex_handle]: https://docs-src.amethyst.rs/stable/amethyst_renderer/type.TextureHandle.html
[doc_component]: https://docs-src.amethyst.rs/stable/specs/trait.Component.html
[doc_handle]: https://docs-src.amethyst.rs/stable/amethyst_assets/struct.Handle.html
[doc_flipped]: https://docs-src.amethyst.rs/stable/amethyst_renderer/struct.Flipped.html
