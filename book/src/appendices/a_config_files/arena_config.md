# Adding an Arena Config

To begin with, let's make the `Arena` dimensions configurable. Add this structure to a new file `config.rs`.

```rust,ignore
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
```

The default values match the values used in the full example, so if we don't use a config file things will 
look just like the Pong example. Another option would be to use [`[#serde(default)]`][serde_default], which allows
you to set the default value of a field if that field is not present in the config file. This is different
than the [`Default`][default] trait in that you can set default values for some fields while requiring others
be present. For now though, let's just use the `Default` trait.

## Adding the Config to the World

Now, in `bundle.rs`, add the following lines:

```rust,ignore
use std::path::Path;

use config::ArenaConfig;
```

In `bundle.rs`, modify the `PongBundle` struct to now have a `config` field.

```rust,ignore
struct PongBundle {
    config: ArenaConfig,
}
```

We'll need to load the config at startup, so let's add a `new()` function that takes the path to the RON 
config file.

```rust,ignore
impl PongBundle {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        PongBundle {
            config: PongConfig::load(path),
        }
    }
}
```

Now that our `PongBundle` knows about our config, we want to add it to the world so other modules can access 
it. We do this by modifying the [`build()`][ecsbuild] function implemented as part of the 
[`ECSBundle`][ecsbundle] trait. Add the following line to the top of the function:

```rust,ignore
world.add_resource(self.config);
```

Now for the difficult part: replacing every use of `ARENA_WIDTH` and `ARENA_HEIGHT` with our config object. 
First, let's change our initialisation steps in `pong.rs`.

Add the following line to the top of `pong.rs`:

```rust,ignore
use config::ArenaConfig;
```

Now, in the `initialise_paddles()` function, add the following lines after the initialisation of the 
`left_transform` and `right_transform`.

```rust,ignore
let (arena_height, arena_width) = {
    let config = &world.read_resource::<ArenaConfig>();
    (config.height, config.width)
};
```

Now replace all references to `ARENA_HEIGHT` with `arena_height` and all references to `ARENA_WIDTH` with 
`arena_width`. Do this for each initialisation function in `pong.rs`.

## Accessing Config Files from Systems

It is actually simpler to access a Config file from a system than via the `World` directly. To access 
it in the `System`'s `run()` function, add it to the `SystemData` type. This is what the `BounceSystem` looks 
like when it wants to access the `ArenaConfig`.

```rust,ignore
use config::ArenaConfig;
...
type SystemData = (
    WriteStorage<'s, Ball>,
    ReadStorage<'s, Paddle>,
    ReadStorage<'s, Transform>,
    Fetch<'s, AssetStorage<Source>>,
    Fetch<'s, Sounds>,
    Fetch<'s, Option<Output>>,
    Fetch<'s, ArenaConfig>,
);
...
fn run(&mut self, 
       (mut balls, paddles, transforms, storage, sounds, audio_output, arena_config): SystemData) {
```

Now, in the `run()` function, replace the reference to `ARENA_HEIGHT` with `arena_config.height`.

Add `Fetch<'s, ArenaConfig>` to the `WinnerSystem` and `PaddleSystem` as well, replacing the reference to 
`ARENA_WIDTH` with `arena_config.width`.

## Making `config.ron`

Now for the final part: actually creating our `config.ron` file. This will be very simple right now, and 
expand as we add more configurable items. For now, just copy and paste the following into a new file. Feel 
free to modify the height and width if you want.

```ignore
arena: (
    height: 100.0,
    width: 100.0,
)
```

[Click here to continue to the next chapter][1]

[config]: https://docs.rs/amethyst_config/0.5.0/amethyst_config/trait.Config.html
[ecsbundle]: https://docs.rs/amethyst_core/0.1.0/amethyst_core/bundle/trait.ECSBundle.html
[ecsbuild]: https://docs.rs/amethyst_core/0.1.0/amethyst_core/bundle/trait.ECSBundle.html#tymethod.build
[1]: ./appendices/a_config_files/ball_config.html
[serde_default]: https://serde.rs/attr-default.html
[default]: https://doc.rust-lang.org/std/default/trait.Default.html
