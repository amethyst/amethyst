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
    }
};
# fn main() -> Result<(), amethyst::Error> {
#
# let game_data = DispatcherBuilder::default()
#     .with_bundle(
#
// inside your rendering bundle setup
RenderingBundle::<DefaultBackend>::new()
    .with_plugin(RenderFlat2D::default())

# )?;
# Ok(()) }
```
