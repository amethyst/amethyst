# Set Up The Render plugin

Amethyst supports drawing sprites using the `RenderFlat2D` render plugin.
To enable this you have to do the following:

```rust
use amethyst::{
    ecs::World,
    prelude::*,
    renderer::{plugins::RenderFlat2D, types::DefaultBackend, RenderingBundle},
};
# fn main() -> Result<(), amethyst::Error> {
#   let game_data = DispatcherBuilder::default().add_bundle(
// inside your rendering bundle setup
#)?;
#   Ok(())
# }
```
