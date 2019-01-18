# Set Up The Render Pass

Amethyst supports drawing sprites using the `DrawFlat2D` render pass. To enable this you have to do the following:

1. Build a `Pipeline` with the `DrawFlat2D` pass. If your sprites have transparent pixels use the `.with_transparency(..)` method.
2. Use the `.with_sprite_sheet_processor()` method on the `RenderBundle`.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
# use amethyst::core::transform::TransformBundle;
# use amethyst::input::InputBundle;
# use amethyst::prelude::*;
use amethyst::renderer::{
    ColorMask, DisplayConfig, DrawFlat2D, Pipeline, RenderBundle, Stage, ALPHA,
};
# use amethyst::ui::UiBundle;
# use amethyst::utils::application_root_dir;
#
# #[derive(Debug, Default)]
# struct ExampleState;
#
# impl SimpleState for ExampleState {}

fn main() -> amethyst::Result<()> {
#     amethyst::start_logger(Default::default());
#     let app_root = application_root_dir()?;
#     let display_config = DisplayConfig::load(app_root.join("examples/sprites/resources/display_config.ron"));
#
    // ...

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0., 0., 0., 1.], 1.)
            .with_pass(
                DrawFlat2D::new()
                    .with_transparency(ColorMask::all(), ALPHA, None)),
    );

    let game_data = GameDataBuilder::default()
#         .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderBundle::new(pipe, Some(display_config))
                .with_sprite_sheet_processor())?

#         .with_bundle(InputBundle::<String, String>::new())?
#         .with_bundle(UiBundle::<String, String>::new())?;
    // ...
#     let assets_directory = app_root.join("examples/assets/");
#     let mut game = Application::new(assets_directory, ExampleState::default(), game_data)?;
#     game.run();
#
#     Ok(())
}
```
