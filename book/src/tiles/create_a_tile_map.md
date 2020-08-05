# Create a Tile Map

With the `tiles` feature installed and our `RenderTiles2D` render pass setup, we can create a `TileMap` component and add it an entity. We need to have a sprite sheet loaded before the creation so this example assume a handle to a sprite sheet exists.

```rust,edition2018,no_run,noplaypen
# extern crate amethyst;
use amethyst::{
    core::{
        math::Vector3,
        transform::Transform,
    },
    tiles:TileMap,
};
# use amethyst::{
#     assets::Handle,
#     prelude::*
#     renderer::SpriteSheet,
# };

# #[derive(Clone, Default)]
# struct SimpleTile;
# impl Tile for SimpleTile {
#     fn sprite(&self, _coords: Point3<u32>, _: &World) -> Option<usize> {
#         Some(1)
#     }
# }

# pub fn load_sprite_sheet() -> Handle<SpriteSheet> {
#     unimplemented!();
# }

#[derive(Debug)]
struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
#        let world = data.world;
        let sprite_sheet_handle = load_sprite_sheet();
        // ...
        init_map(world, sprite_sheet_handle.clone())
    }
}

fn init_map(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
        let map = TileMap::<SimpleTile>::new(
            Vector3::new(10, 10, 1), // The dimensions of the map
            Vector3::new(16, 16, 1), // The dimensions of each tile
            Some(sprite_sheet_handle),
        );
        let transform = Transform::default();
        
        world
            .create_entity()
            .with(map)
            .with(transform)
            .build();
}
#
# fn main() {}
```

The tile map component was created and added to the entity we created and thats it! Check out the [*tiles*][ex_tiles] example in the [examples][ex_all] directory.

[ex_tiles]: https://github.com/amethyst/amethyst/tree/master/examples/tiles
[ex_all]: https://github.com/amethyst/amethyst/tree/master/examples
