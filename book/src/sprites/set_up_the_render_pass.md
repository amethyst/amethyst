# Set Up The Render plugin

Amethyst supports drawing sprites using the `RenderFlat2D` render plugin.
To enable this you have to do the following:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
#
use amethyst::{
    ecs::{World, WorldExt},
    prelude::*,
    renderer::{
        plugins::RenderFlat2D,
        types::DefaultBackend,
        RenderingBundle,
    },
    window::{DisplayConfig, EventLoop},
    utils::application_root_dir,
};
# fn main() -> Result<(), amethyst::Error> {
# let app_root = application_root_dir()?;
#
# let display_config_path = app_root.join("config").join("display.ron");
#
# let event_loop = EventLoop::new();
# let display_config = DisplayConfig::load(display_config_path)?;
# let game_data = GameDataBuilder::default()
#     .with_bundle(
#
// inside your rendering bundle setup
RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)
    .with_plugin(RenderFlat2D::default())

# )?;
# Ok(()) }
```
