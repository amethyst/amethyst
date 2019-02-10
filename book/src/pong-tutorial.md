# Pong Tutorial

To get a better feeling for how Amethyst works, we're going to implement a 
Pong clone. You can find a [full Pong example][pong] (our end goal) in Amethyst's 
examples folder. This tutorial breaks that project up into discrete steps so 
it's easier to understand what everything is doing. If you've cloned the 
Amethyst repo, you can run any of the examples like so:

```norun
cargo run --example pong_tutorial_01
```

The main difference between real game code and the example code is where the 
`resources` and `assets` folders are located.

For instance, in the pong_tutorial_01 example we have:

```rust,ignore
let path = application_dir("examples/pong_tutorial_01/resources/display_config.ron")?;
```

But for your own project you'll probably want something like this:

```rust,ignore
let path = "./resources/display_config.ron";
```

[pong]: https://github.com/amethyst/amethyst/tree/master/examples/pong

