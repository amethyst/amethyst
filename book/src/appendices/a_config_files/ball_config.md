# Adding a Ball Config

For simplicity, we will wrap our Config objects into a single `PongConfig` object backed by a single
`config.ron` file, but know that you can keep them in separate files and read from each file
separately.

To prepare for our `BallConfig`, add the following line to the top of `config.rs`:

```rust
use amethyst::core::math::Vector2;
```

The `BallConfig` will replace the `BALL_VELOCITY_X`, `BALL_VELOCITY_Y`, `BALL_RADIUS`, and `BALL_COLOR`
variables. We'll use a [`Vector2`][vec2] to store the velocity for simplicity and to demonstrate how to add
a non-trivial data type to a RON file. The `BALL_COLOR` was originally an array, but [Serde] and RON
handle arrays as tuples, so it will read in a tuple and convert the color values to an array if needed by a
particular function (e.g., in `pong.rs`).

```rust
use amethyst::core::math::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BallConfig {
    pub velocity: Vector2<f32>,
    pub radius: f32,
    pub color: (f32, f32, f32, f32),
}
```

We'll also add the `Default` trait to this config that will match what the full example uses.

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

impl Default for BallConfig {
    fn default() -> Self {
        BallConfig {
            velocity: Vector2::new(75.0, 50.0),
            radius: 2.5,
            color: (1.0, 0.0, 0.0, 1.0),
        }
    }
}
```

Still in `config.rs`, add the following structure definition at the bottom. This structure will be
backed by the whole `config.ron` file.

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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PongConfig {
    pub arena: ArenaConfig,
    pub ball: BallConfig,
}
```

## Replacing Ball Constants

Now we need to replace our usage of the `BALL_*` constants with our new `BallConfig`.

We use these values in `pong.rs` in the `initialize_ball()` function, so the substitution is even simpler than
the `ArenaConfig`.

In `pong.rs`, underneath our loading of the `ArenaConfig`, add the following lines

```rust
# use amethyst::core::math::Vector2;
# use amethyst::ecs::Resources;
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
# fn main() -> amethyst::Result<()> {
#   let mut resources = Resources::default();
#   resources.insert(BallConfig::default());
    let (velocity_x, velocity_y, radius, color) = {
        let config = resources.get::<BallConfig>().unwrap();
        let c: [f32; 4] = [
            config.color.0,
            config.color.1,
            config.color.2,
            config.color.3,
        ];
        (config.velocity.x, config.velocity.y, config.radius, c)
    };
#   Ok(())
# }
```

Our functions expect a `[f32; 4]` array, so we had to convert the tuple to an array. This is
simple to do, but for more complex arrays it might be worth it to add a function to the `impl BallConfig` to
avoid duplicating this effort.

Now, within the `initialize_ball` function, replace `BALL_VELOCITY_X` with `velocity_x`, `BALL_VELOCITY_Y`
with `velocity_y`, `BALL_RADIUS` with `radius`, and `BALL_COLOR` with `color`.

## Modifying the initialization

Now we will modify our application initialization. We don't want everyone to always access all the config files, so we need to
add each resource separately so systems can use only what they want.

First, we need to change what `main.rs` is using. Change

```rust ,ignore
use crate::config::ArenaConfig;
```

to

```rust ,ignore
use crate::config::PongConfig;
```

Now, modify the `main` function, from

```rust
# use amethyst::{
#   assets::LoaderBundle,
#   config::Config,
#   ecs::{DispatcherBuilder, ParallelRunnable},
#   Application, EmptyState,
# };
# 
# mod config {
#   use serde::{Deserialize, Serialize};
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
# }
# 
# struct NullState;
# 
# impl EmptyState for NullState {}
# 
# fn main() -> amethyst::Result<()> {
    let arena_config = config::ArenaConfig::load("config.ron");

#   Application::build("", NullState)?
        // ..
        .with_resource(arena_config);

#   Ok(())
# }
```

to

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
#   #[derive(Debug, Default, Deserialize, Serialize)]
#   pub struct PongConfig {
#       pub arena: ArenaConfig,
#       pub ball: BallConfig,
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
        .with_resource(pong_config.ball);

#   Ok(())
# }
```

## Adding the BallConfig to `config.ron`

Now we need to modify our configuration file to allow multiple structures to be included. This is
easy with RON; we add an additional level of nesting.

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
)
```

This configuration sets the ball to be orange, while retaining the same size and velocity as the original
example.

[serde]: https://docs.serde.rs/serde/index.html
[vec2]: https://docs.rs/nalgebra/0.24.0/nalgebra/base/type.Vector2.html
