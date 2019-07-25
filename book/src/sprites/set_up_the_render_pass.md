# Set Up The Render plugin

Amethyst supports drawing sprites using the `RenderFlat2D` render plugin.
To enable this you have to do the following:

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
# use amethyst::{prelude::*};
use amethyst::renderer::{plugin::RenderFlat2D, RenderingBundle};

# fn main() -> Result<(), amethyst::Error>{
#let game_data = GameDataBuilder::default()
#.with_bundle(
// inside your rendering bundle setup
RenderingBundle::<DefaultBackend>::new()
    .with_plugin(RenderFlat2D::default())
#)?;
# Ok(()) }
```
