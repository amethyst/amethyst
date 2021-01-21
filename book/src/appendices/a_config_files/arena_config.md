# Adding an Arena Config

To begin with, let's make the `Arena` dimensions configurable. Add this structure to a new file `config.rs`.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ArenaConfig {
    pub height: f32,
    pub width: f32,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        ArenaConfig {
            height: 100.0,
            width: 100.0,
        }
    }
}

fn main() {}
```

The default values match the values used in the full example, so if we don't use a config file things will
look just like the Pong example. Another option would be to use [`[#serde(default)]`][serde_default], which allows
you to set the default value of a field if that field is not present in the config file. This is different
than the [`Default`][default] trait in that you can set default values for some fields while requiring others
be present. For now though, let's just use the `Default` trait.

## Adding the Config to the World

We'll need to load the config at startup, so let's add this to the `run` function in `main.rs`

```rust
# use serde::{Deserialize, Serialize};
# 
# #[derive(Debug, Deserialize, Serialize)]
# struct ArenaConfig {
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
# fn main() {
    let arena_config = ArenaConfig::default();
# }
```

Now that we have loaded our config, we want to add it to the world so other modules can access
it. We do this by adding the config as a resource during `Application` creation:

```rust
# use serde::{Deserialize, Serialize};
# 
# #[derive(Debug, Deserialize, Serialize)]
# struct ArenaConfig {
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

struct NullState;

impl EmptyState for NullState {}

fn main() {
#   let arena_config = ArenaConfig::default();
# 
    use amethyst::{
        assets::LoaderBundle,
        ecs::{DispatcherBuilder, ParallelRunnable},
    };
    let mut builder = DispatcherBuilder::default();
    builder.add_bundle(LoaderBundle);

    Application::build("", NullState).with_resource(arena_config);
}
```

Now for the difficult part: replacing every use of `ARENA_WIDTH` and `ARENA_HEIGHT` with our config object.
First, let's change our initialisation steps in `pong.rs`.

Add the following line to the top of `pong.rs`:

```rust ,ignore
use crate::config::ArenaConfig;
```

Now, in the `initialise_paddles()` function, add the following lines after the initialisation of the
`left_transform` and `right_transform`.

```rust
# use amethyst::StateData;
# 
# fn main() {
#   let data = StateData::new(World::default(), Resources::default, ());
#   let StateData { resources, .. } = data;
    let (arena_height, arena_width) = {
        let config = &resources.get::<ArenaConfig>();
        (config.height, config.width)
    };
# }
```

Now replace all references to `ARENA_HEIGHT` with `arena_height` and all references to `ARENA_WIDTH` with
`arena_width`. Do this for each initialisation function in `pong.rs`.

## Accessing Config Files from Systems

It is actually simpler to access a Config file from a system than via the `World` directly. To access
it in the `System`'s `run()` function, add it to the `SystemData` type. This is what the `BounceSystem` looks
like when it wants to access the `ArenaConfig`.

```rust
use amethyst::ecs::{Resources, System, SystemBuilder};

use crate::config::ArenaConfig;

struct ArenaSystem;

impl System<'_> for ArenaSystem {
    fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("ArenaSystem")
                .read_resource::<ArenaConfig>()
                .build(|_, _, arena_config, _| {
                    println!("{} x {}", arena_config.width, arena_config.height)
                }),
        )
    }
}
```

Now, in the `run()` function, replace the reference to `ARENA_HEIGHT` with `arena_config.height`.

Add `Read<'s, ArenaConfig>` to the `WinnerSystem` and `PaddleSystem` as well, replacing the reference to
`ARENA_WIDTH` with `arena_config.width`.

## Making `config.ron`

Now for the final part: actually creating our `config.ron` file. This will be very simple right now, and
expand as we add more configurable items. For now, just copy and paste the following into a new file. Feel
free to modify the height and width if you want.

```ron
arena: (
    height: 100.0,
    width: 100.0,
)
```

[default]: https://doc.rust-lang.org/std/default/trait.Default.html
[serde_default]: https://serde.rs/attr-default.html
