# Appendix A: Config Files

In the [full Pong example][pong], the paddle sizes, ball sizes, colors, and arena size are all hard-coded 
into the implementation. This means that if you want to change any of these, you need to recompile the 
project. Wouldn't it be nice to not have to recompile the project each time you wanted to change one or all 
of these things?

Luckily, Amethyst uses [RON][ron] configuration files and has infrastructure in the form of the 
[Config][config] trait to help us implement our own config files.

## Structure of the Config File

The existing example uses the following constants:

```rust,ignore
const ARENA_HEIGHT: f32 = 100.0;
const ARENA_WIDTH: f32 = 100.0;
const PADDLE_HEIGHT: f32 = 15.0;
const PADDLE_WIDTH: f32 = 2.5;
const PADDLE_VELOCITY: f32 = 75.0;
const PADDLE_COLOUR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

const BALL_VELOCITY_X: f32 = 75.0;
const BALL_VELOCITY_Y: f32 = 50.0;
const BALL_RADIUS: f32 = 2.5;
const BALL_COLOUR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
```

to specify the look of the game. We want to replace this with something more flexible in the form of a config 
file. To start, let's create a new file, `config.rs`, to hold our configuration structures. Add the following 
`use` statements to the top of this file:

```rust,ignore
use std::path::Path;

use amethyst::config::Config;
```

For this project, we'll be placing a `config.ron` file in the same location as the `display.ron` and 
`input.ron` files (likely the `config/` folder).

## Chapters

* [Adding an ArenaConfig][0]
* [Adding a Ball Config][1]
* [Adding Paddle Configs][2]

[pong]: https://github.com/amethyst/amethyst/tree/master/examples/pong
[ron]: https://docs.rs/ron/0.3.0/ron/
[config]: https://docs.rs/amethyst_config/0.6.0/amethyst_config/trait.Config.html
[0]: ./a_config_files/arena_config.html
[1]: ./a_config_files/ball_config.html
[2]: ./a_config_files/paddle_configs.html

