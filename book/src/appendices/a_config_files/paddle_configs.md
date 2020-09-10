# Adding Paddle Configs

We're finally going to add a configuration struct for our Paddles. Because our Pong clone supports two 
players, we should let them configure each separately. Add the following to the `config.rs` file:

```rust,ignore
#[derive(Debug, Deserialize, Serialize)]
pub struct PaddleConfig {
    pub height: f32,
    pub width: f32,
    pub velocity: f32,
    pub color: (f32, f32, f32, f32),
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

Just like the `BallConfig`, we need to read in the color as a tuple instead of an array.

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

and modify the `main.rs`'s `run()` function to add our `PaddleConfig`s. 

```rust,ignore
    .with_resource(pong_config.arena)
    .with_resource(pong_config.ball)
    .with_resource(pong_config.paddles)
    .with_bundle(PongBundle::default())?
```

We add the `PaddlesConfig` to the `World`, rather than as separate `left` and `right` configurations because
`System`s can only access resources with ID 0. Any resource added using `World::add_resource`
is added using a default ID of 0. You must use `World::add_resource_with_id` to add multiple
resources of the same type, but then the `System`s cannot properly differentiate between them.

## Replacing Constants with Configs

Replacing all instances of `PADDLE_*` will be similar to the `BallConfig`, as we only use those values for 
creating the paddle entities. However, we will need to separate the `PaddlesConfig` into `left` and `right`.
To avoid issues with the borrow checker, we read the `PaddlesConfig` once and copy all of the values, 
unwrapping them in one big assignment statement.
In `initialise_paddles()` in `pong.rs`, add this code below reading the `ArenaConfig`.

```rust,ignore
let (
    left_height,
    left_width,
    left_velocity,
    left_color,
    right_height,
    right_width,
    right_velocity,
    right_color,
) = {
    let config = &world.read_resource::<PaddlesConfig>();
    let cl: [f32; 4] = [
        config.left.color.0,
        config.left.color.1,
        config.left.color.2,
        config.left.color.3,
    ];
    let cr: [f32; 4] = [
        config.right.color.0,
        config.right.color.1,
        config.right.color.2,
        config.right.color.3,
    ];
    (
        config.left.height,
        config.left.width,
        config.left.velocity,
        cl,
        config.right.height,
        config.right.width,
        config.right.velocity,
        cr,
    )
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
`create_color_material()` so that you have a left and right mesh and left
and right color.

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
        velocity: Vector2(
            x: 75.0,
            y: 50.0,
        ),
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