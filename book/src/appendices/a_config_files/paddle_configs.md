# Adding Paddle Configs

We're finally going to add a configuration struct for our Paddles. Because our Pong clone supports two 
players, we should let them configure each separately. Add the following to the `config.rs` file:
```rust,ignore
#[derive(Debug, Deserialize, Serialize)]
pub struct PaddleConfig {
    pub height: f32,
    pub width: f32,
    pub velocity: f32,
    pub colour: (f32, f32, f32, f32),
}

impl Default for PaddleConfig {
    fn default() -> Self {
        PaddleConfig {
            height: 15.0,
            width: 2.5,
            velocity: 75.0,
            color: (0.0, 0.0, 1.0, 1.0),
        }
    }
}
```

Just like the `BallConfig`, we need to read in the colour as a tuple instead of an array.

Now, to allow us to have two separate `PaddleConfig`s, we will wrap them in a bigger structure as follows:
```rust,ignore
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PaddlesConfig {
    pub left: PaddleConfig,
    pub right: PaddleConfig,
}
```

Now we need to add the `PaddlesConfig` to our `PongConfig` as shown below
```rust,ignore
pub struct PongConfig {
    pub arena: ArenaConfig,
    pub ball: BallConfig,
    pub paddles: PaddlesConfig,
}
```
and modify the `ECSBundle`'s `build()` function to add our `PaddleConfig`s. 
```rust,ignore
fn build(
    self,
    world: &mut World,
    builder: DispatcherBuilder<'a, 'b>,
) -> Result<DispatcherBuilder<'a, 'b>> {
    world.add_resource(self.config.arena);
    world.add_resource(self.config.ball);
    world.add_resource_with_id(self.config.paddles.left, 0);
    world.add_resource_with_id(self.config.paddles.right, 1);
    ...
}
```
The one change from the previous examples is that we need to add resources with an ID. Previous configs only 
had one resource per type, so it was okay to add them with the default ID of 0 (as 
[`World::add_resource`][add_resource] does). Thus, we use [`World::add_resource_with_id`][add_with_id] 
instead.

## Replacing Constants with Configs
Replacing all instances of `PADDLE_*` will be similar to the `BallConfig`, as we only use those values for 
creating the paddle entities. However, we will need to separate the `PaddlesConfig` into `left` and `right`.
In `initialise_paddles()` in `pong.rs`, add this code below reading the `ArenaConfig`.
```rust,ignore
let (left_height, left_width, left_velocity, left_color) = {
    let config = &world.read_resource_with_id::<PaddleConfig>(0);
    let c: [f32; 4] = [
        config.color.0,
        config.color.1,
        config.color.2,
        config.color.3,
    ];
    (config.height, config.width, config.velocity, c)
};
let (right_height, right_width, right_velocity, right_color) = {
    let config = &world.read_resource_with_id::<PaddleConfig>(1);
    let c: [f32; 4] = [
        config.color.0,
        config.color.1,
        config.color.2,
        config.color.3,
    ];
    (config.height, config.width, config.velocity, c)
};
```
Now, within this function, replace
```rust,ignore
let y = (arena_height - PADDLE_HEIGHT) / 2.0;
```
with 
```rust,ignore
let left_y = (arena_height - left_height) / 2.0;
let right_y = (arena_height - right_height) / 2.0;
```

You will also need to repeat the calls to `create_mesh` and 
`create_colour_material()` such that you have a left and right mesh and left
and right colour.

Now, use the left- and right-specific values in  the `world.create_entity()` 
calls.

## Modifying `config.ron`
Now for the final modification of our `config.ron` file. For fun, let's make the right paddle yellow and
keep the left paddle blue so the final `config.ron` file will be as follows:
```ignore
(
    arena: (
        height: 100.0,
        width: 100.0,
    ),
    ball: (
        velocity_x: 75.0,
        velocity_y: 50.0,
        radius: 2.5,
        color: (1.0, 0.647, 0.0, 1.0),
    ),
    paddles: (
        left: (
            height: 15.0,
            width: 2.5,
            velocity: 75.0,
            color: (0.0, 0.0, 1.0, 1.0),
        ),
        right: (
            height: 15.0,
            width: 2.5,
            velocity: 75.0,
            color: (0.0, 1.0, 1.0, 1.0),
        ),
    )
)
```


[add_resource]: https://docs.rs/specs/0.10.0/specs/struct.World.html#method.add_resource
[add_with_id]: https://docs.rs/specs/0.10.0/specs/struct.World.html#method.add_resource_with_id


