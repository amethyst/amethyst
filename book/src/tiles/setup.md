# Setup

## Install the tiles feature

In order to use the tiles package you need add the `tiles` feature to your `Cargo.toml`:

```rust
[dependencies]
amethyst = { version = "LATEST_CRATES.IO_VERSION", features = ["tiles"] }
```

## Setup the Render Pass

Now you can add the render pass to your application:

```rust
use amethyst::{
    core::math::Point3,
    ecs::World,
    prelude::*,
    renderer::{plugins::RenderFlat2D, types::DefaultBackend, RenderingBundle},
    tiles::{RenderTiles2D, Tile},
};

#[derive(Clone, Default)]
struct SimpleTile;
impl Tile for SimpleTile {
    fn sprite(&self, _coords: Point3<u32>, _: &World) -> Option<usize> {
        Some(1)
    }
}

# fn main() -> Result<(), amethyst::Error> {
#   let game_data = DispatcherBuilder::default().add_bundle(
        // inside your rendering bundle setup
        RenderingBundle::<DefaultBackend>::new().with_plugin(RenderFlat2D::default()),
#   )?;
#   Ok(())
# }
```

The render plugin requires a tile implementation, so we create a struct, `SimpleTile` and implement `Tile`, which is needed by the render plugin in order to provide the sprite number and tint (not implemented in this example) to the renderer. The tile we created will also be used later when we create the tile map.
