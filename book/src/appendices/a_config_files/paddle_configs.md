# Adding Paddle Configs

Finally, we're going to add a configuration struct for our Paddles. Because our Pong clone supports two
players, we should let them configure each separately. Add the following to the `config.rs` file:

```rust
# use serde::{Deserialize, Serialize};

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

```rust
# use serde::{Deserialize, Serialize};
# 
# #[derive(Debug, Deserialize, Serialize)]
# pub struct PaddleConfig {
#   pub height: f32,
#   pub width: f32,
#   pub velocity: f32,
#   pub color: (f32, f32, f32, f32),
# }
# 
# impl Default for PaddleConfig {
#   fn default() -> Self {
#       PaddleConfig {
#           height: 15.0,
#           width: 2.5,
#           velocity: 75.0,
#           color: (0.0, 0.0, 1.0, 1.0),
#       }
#   }
# }

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PaddlesConfig {
    pub left: PaddleConfig,
    pub right: PaddleConfig,
}
```

Now we need to add the `PaddlesConfig` to our `PongConfig` as shown below

```rust
# use amethyst::core::math::Vector2;
# use serde::{Deserialize, Serialize};
# 
# #[derive(Debug, Deserialize, Serialize)]
# pub struct BallConfig {
#   pub velocity: Vector2<f32>,
#   pub radius: f32,
#   pub color: (f32, f32, f32, f32),
# }
# 
# impl Default for BallConfig {
#   fn default() -> Self {
#       BallConfig {
#           velocity: Vector2::new(75.0, 50.0),
#           radius: 2.5,
#           color: (1.0, 0.0, 0.0, 1.0),
#       }
#   }
# }
# 
# #[derive(Debug, Deserialize, Serialize)]
# pub struct ArenaConfig {
#   pub height: f32,
#   pub width: f32,
# }
# 
# impl Default for ArenaConfig {
#   fn default() -> Self {
#       ArenaConfig {
#           height: 100.0,
#           width: 100.0,
#       }
#   }
# }
# 
# #[derive(Debug, Deserialize, Serialize)]
# pub struct PaddleConfig {
#   pub height: f32,
#   pub width: f32,
#   pub velocity: f32,
#   pub color: (f32, f32, f32, f32),
# }
# 
# impl Default for PaddleConfig {
#   fn default() -> Self {
#       PaddleConfig {
#           height: 15.0,
#           width: 2.5,
#           velocity: 75.0,
#           color: (0.0, 0.0, 1.0, 1.0),
#       }
#   }
# }
# 
# #[derive(Debug, Default, Deserialize, Serialize)]
# pub struct PaddlesConfig {
#   pub left: PaddleConfig,
#   pub right: PaddleConfig,
# }
# 

#[derive(Debug, Deserialize, Serialize)]
pub struct PongConfig {
    pub arena: ArenaConfig,
    pub ball: BallConfig,
    pub paddles: PaddlesConfig,
}
```

and modify the `main.rs`'s `run()` function to add our `PaddleConfig`s.

```rust
# use amethyst::{
#   assets::LoaderBundle,
#   config::Config,
#   ecs::{DispatcherBuilder, ParallelRunnable},
#   Application, EmptyState,
# };
# 
# mod config {
#   use amethyst::core::math::Vector2;
#   use serde::{Deserialize, Serialize};
# 
#   #[derive(Debug, Deserialize, Serialize)]
#   pub struct BallConfig {
#       pub velocity: Vector2<f32>,
#       pub radius: f32,
#       pub color: (f32, f32, f32, f32),
#   }
# 
#   impl Default for BallConfig {
#       fn default() -> Self {
#           BallConfig {
#               velocity: Vector2::new(75.0, 50.0),
#               radius: 2.5,
#               color: (1.0, 0.0, 0.0, 1.0),
#           }
#       }
#   }
# 
#   #[derive(Debug, Deserialize, Serialize)]
#   pub struct ArenaConfig {
#       pub height: f32,
#       pub width: f32,
#   }
# 
#   impl Default for ArenaConfig {
#       fn default() -> Self {
#           ArenaConfig {
#               height: 100.0,
#               width: 100.0,
#           }
#       }
#   }
# 
#   #[derive(Debug, Deserialize, Serialize)]
#   pub struct PaddleConfig {
#       pub height: f32,
#       pub width: f32,
#       pub velocity: f32,
#       pub color: (f32, f32, f32, f32),
#   }
# 
#   impl Default for PaddleConfig {
#       fn default() -> Self {
#           PaddleConfig {
#               height: 15.0,
#               width: 2.5,
#               velocity: 75.0,
#               color: (0.0, 0.0, 1.0, 1.0),
#           }
#       }
#   }
# 
#   #[derive(Debug, Default, Deserialize, Serialize)]
#   pub struct PaddlesConfig {
#       pub left: PaddleConfig,
#       pub right: PaddleConfig,
#   }
# 
#   #[derive(Debug, Default, Deserialize, Serialize)]
#   pub struct PongConfig {
#       pub arena: ArenaConfig,
#       pub ball: BallConfig,
#       pub paddles: PaddlesConfig,
#   }
# }
# 
# struct NullState;
# 
# impl EmptyState for NullState {}
# 
# fn main() -> amethyst::Result<()> {
    let pong_config = config::PongConfig::load("config.ron").unwrap_or_default();

#   Application::build("", NullState)?
        //..
        .with_resource(pong_config.arena)
        .with_resource(pong_config.ball)
        .with_resource(pong_config.paddles);

#   Ok(())
# }
```

## Replacing Constants with Configs

Replacing all instances of `PADDLE_*` will be similar to the `BallConfig`, as we only use those values for
creating the paddle entities. However, we will need to separate the `PaddlesConfig` into `left` and `right`.
To avoid issues with the borrow checker, we read the `PaddlesConfig` once and copy all of the values,
unwrapping them in one big assignment statement.
In `initialize_paddles()` in `pong.rs`, add this code below reading the `ArenaConfig`.

```rust
# use amethyst::ecs::Resources;
# use serde::{Deserialize, Serialize};
# 
# #[derive(Debug, Deserialize, Serialize)]
# pub struct PaddleConfig {
#   pub height: f32,
#   pub width: f32,
#   pub velocity: f32,
#   pub color: (f32, f32, f32, f32),
# }
# 
# impl Default for PaddleConfig {
#   fn default() -> Self {
#       PaddleConfig {
#           height: 15.0,
#           width: 2.5,
#           velocity: 75.0,
#           color: (0.0, 0.0, 1.0, 1.0),
#       }
#   }
# }
# 
# #[derive(Debug, Default, Deserialize, Serialize)]
# pub struct PaddlesConfig {
#   pub left: PaddleConfig,
#   pub right: PaddleConfig,
# }
# 
# fn main() -> amethyst::Result<()> {
#   let mut resources = Resources::default();
#   resources.insert(PaddlesConfig::default());

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
        let config = resources.get::<PaddlesConfig>().unwrap();
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
#   Ok(())
# }
```

Now, within this function, replace

```rust
# const PADDLE_HEIGHT: f32 = 15.0;
# fn main() {
#   let arena_height = 0.;
    let y = (arena_height - PADDLE_HEIGHT) / 2.0;
# }
```

with

```rust
# fn main() {
#   let arena_height = 0.;
#   let left_height = 0.;
#   let right_height = 0.;
    let left_y = (arena_height - left_height) / 2.0;
    let right_y = (arena_height - right_height) / 2.0;
# }
```

You will also need to repeat the calls to `create_mesh` and
`create_color_material()` so that you have a left and right mesh and left
and right color.

Now, use the left- and right-specific values in  the `world.push()` calls.

## Modifying `config.ron`

Now for the final modification of our `config.ron` file. For fun, let's make the right paddle yellow and
keep the left paddle blue so the final `config.ron` file will be as follows:

```ron
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
