# Pong Tutorial

To get a better feeling for how Amethyst works, we're going to implement a
Pong clone. You can find a [full Pong example][pong] (our end goal) in
Amethyst's [examples] folder. This tutorial breaks that project up into discrete
steps so it's easier to understand what everything is doing.

## Prerequisites

Make sure to follow the [Getting started chapter](./getting-started.html) before
starting with the tutorial / running the examples.

## Running the code after a chapter

If you've cloned the Amethyst repo, you can run any of the examples like so:

```norun
cargo run --example pong_tutorial_01 --features "vulkan"
```

The example named `pong_tutorial_xy` contains the code which you should have
after following all tutorials from 1 to xy.

> **Note:** On macOS, you might want to use `"metal"` instead of `"vulkan"`.

The main difference between real game code and the example code is where the 
`config` and `assets` folders are located.

For instance, in the pong_tutorial_01 example we have:

```rust,ignore
let display_config_path =
    app_root.join("examples/pong_tutorial_01/config/display.ron");

let assets_dir = app_root.join("examples/assets/");
```

But for your own project you'll probably want something like this:

```rust,ignore
let display_config_path = app_root.join("config/display.ron");

let assets_dir = app_root.join("assets/");
```

[pong]: https://github.com/amethyst/amethyst/tree/master/examples/pong_tutorial_06
[examples]: https://github.com/amethyst/amethyst/tree/master/examples

