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
use amethyst::assets::{AssetStorage, Handle, Loader};
use amethyst::core::Transform;
use amethyst::prelude::*;
use amethyst::renderer::{ImageFormat, Texture};
use amethsyt::renderer::resources::Tint;
use amethyst::renderer::palette::rgb::Srgba;

fn init_image(world: &mut World, texture_handle: &Handle<Texture>) {
    use amethyst::core::math::RealField;

    // Add a transform component to give the image a position
    let mut transform = Transform::default();
    transform.set_translation_x(0.0);
    transform.set_translation_y(0.0);
    
    // Flip horizontally
    transform.set_rotation_y_axis(f32::pi());

    // Color white to show as normal, then update later
    let color = Tint(Srgba::new(1.0, 1.0, 1.0, 1.0));

    world
        .create_entity()
        .with(transform)
        .with(color)
        .with(texture_handle.clone())
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

[doc_tint]: https://docs-src.amethyst.rs/stable/amethyst_rendy/resources/struct.Tint.html
[doc_component]: https://docs-src.amethyst.rs/stable/specs/trait.Component.html
